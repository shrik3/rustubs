//! Driver for the PS/2 keybard/mouse controller
//!
//! beyound OOStuBS:
//!
//! The 'gather' field should NEVER have imcomplete state: it should either be a
//! valid key, or nothing.
//! TODO what if IO ops fails or timeout?
//! TODO figure out how to abstract the "interrupt control" layer.
//! Keyboard controller should not be arch dependent
//!

use self::super::key::*;
use crate::arch::x86_64::is_int_enabled;
use crate::io::*;
use crate::machine::device_io::*;
use crate::proc::sync::semaphore::{Semaphore, SleepSemaphore};
use crate::proc::sync::IRQHandlerEpilogue;
use crate::proc::sync::{L3GetRef, L3SyncCell};
use alloc::collections::VecDeque;
use bitflags::bitflags;
use core::cmp::{Eq, PartialEq};
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::interrupt::{pic_8259, pic_8259::PicDeviceInt as PD};

use super::key::Modifiers;

/// this is the global keybard controller. It can only be used in prologue level,
/// except for the `gather` field (consume_key is safe), which is atomic that can be safely used.
/// (TODO maybe we can change this L3/L2 shared state abstraction)
pub static KBCTL_GLOBAL: L3SyncCell<KeyboardController> =
	L3SyncCell::new(KeyboardController::new());
pub static KEY_BUFFER: SleepSemaphore<VecDeque<Key>> = SleepSemaphore::new(VecDeque::new());

pub struct KeyboardController {
	keystate: KeyState,
	/// gather require synchronization between L2 and L3, therefore we pack the
	/// Key into a u32 and use an Atomic type here.
	gather: AtomicU32,
	cport: IOPort,
	dport: IOPort,
}

/// we have a global controller, but we can't use it as the "driver" because
/// drivers should be stateless.

pub struct KeyboardDriver {}

impl IRQHandlerEpilogue for KeyboardDriver {
	unsafe fn do_prologue() {
		assert!(!is_int_enabled());
		KBCTL_GLOBAL.l3_get_ref_mut().fetch_key();
	}
	unsafe fn do_epilogue() {
		assert!(is_int_enabled());
		let k = KBCTL_GLOBAL.l3_get_ref_mut().consume_key();
		if k.is_some() {
			// this is not ideal. starvation can happen if a thread is
			// intentionally holding the lock forever. This could also starve
			// the other threads because context swap can only happen in-between
			// epilogue execution.
			KEY_BUFFER.v(k.unwrap());
		}
	}
}

struct KeyState {
	modi: Modifiers, // active modifiers
	prefix: Prefix,  // prefix state for some certain modifiers
	scan: Option<u8>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Prefix {
	PREFIX1,
	PREFIX2,
	NONE, // don't confuse Option None with Prefix enum NONE
}

impl Prefix {
	pub fn try_from_u8(val: u8) -> Option<Prefix> {
		match val {
			Defs::C_PREFIX1 => Some(Self::PREFIX1),
			Defs::C_PREFIX2 => Some(Self::PREFIX2),
			_ => None,
		}
	}
}

impl KeyState {
	pub const fn new() -> Self {
		Self {
			modi: Modifiers::NONE,
			prefix: Prefix::NONE,
			scan: None,
		}
	}

	pub fn get_leds(&self) -> u8 {
		return self.modi.bits() & 0b111;
	}
}

impl KeyboardController {
	pub const fn new() -> Self {
		Self {
			keystate: KeyState::new(),
			cport: IOPort::new(Defs::CTRL),
			dport: IOPort::new(Defs::DATA),
			gather: AtomicU32::new(Key::NONE_KEY),
		}
	}

	// TODO: rollback lock state if setting led fails
	fn toggle_lock(&mut self, lock: Modifiers) {
		self.keystate.modi.toggle(lock);
		self.update_led();
	}
	/// in some corner cases, e.g. keyboard control I/O may fail, while the
	/// CAPSLOCK is set and the uppcase table is used for key decoding. So we
	/// never set the leds explicitly, instead we update the leds from the
	/// current keyboard state i.e. keystate.modi
	fn update_led(&self) {
		let leds = self.keystate.get_leds();
		// TODO perhaps disable interrupts here
		// TODO set a timeout. The ACK reply may never come
		// 1. write command
		self.dport.outb(Cmd::SetLed as u8);
		unsafe { self.__block_until_data_available() }
		// 2. wait for ack
		let reply = self.dport.inb();
		// 3. write leds
		if reply == Msg::ACK as u8 {
			self.dport.outb(leds);
		}
		// we should wait for another ACK but we don't really care.
		// just make sure we wait...
		unsafe {
			self.__block_until_data_available();
		}
	}

	pub fn update_state(&mut self, code: u8) {
		// TODO investigate this code pattern: is there much runtime cost??
		self.keystate.scan = Some(code);
		if let Some(p) = Prefix::try_from_u8(code) {
			self.keystate.prefix = p;
			return;
		}
		if code & Defs::BREAK_BIT == 0 {
			if self.press_event() {
				self.decode_key();
			}
		} else {
			self.release_event();
		}
		// the prefix should have been comsumed at this point so clear it
		self.keystate.prefix = Prefix::NONE;
	}
	// TODO: need a rewrite for the toggle_lock and toggle_led
	fn press_event(&mut self) -> bool {
		let mut should_decode_ascii = false;
		let code = self.keystate.scan.unwrap();
		match code {
			Defs::C_SHIFT_L | Defs::C_SHIFT_R => self.keystate.modi.insert(Modifiers::SHIFT),
			Defs::C_ALT => match self.keystate.prefix {
				Prefix::PREFIX1 => self.keystate.modi.insert(Modifiers::ALT_RIGHT),
				_ => self.keystate.modi.insert(Modifiers::ALT_LEFT),
			},
			Defs::C_CTRL => match self.keystate.prefix {
				Prefix::PREFIX1 => self.keystate.modi.insert(Modifiers::CTRL_RIGHT),
				_ => self.keystate.modi.insert(Modifiers::CTRL_LEFT),
			},
			Defs::C_CAPSLOCK => self.toggle_lock(Modifiers::CAPSLOCK),
			Defs::C_NUM_P => {
				if !self.keystate.modi.contains(Modifiers::CTRL_LEFT) {
					self.toggle_lock(Modifiers::NUMLOCK);
				}
			}
			Defs::C_SCRLOCK => self.toggle_lock(Modifiers::SCROLL_LOCK),
			Defs::C_DEL => {
				if self
					.keystate
					.modi
					.contains(Modifiers::CTRL_LEFT | Modifiers::ALT_LEFT)
				{
					unsafe {
						self.reboot();
					}
				}
			}
			_ => {
				should_decode_ascii = true;
			}
		}
		should_decode_ascii
	}

	fn release_event(&mut self) {
		// we only care about release events for shift/alt/ctrl
		let code = self.keystate.scan.unwrap() & !Defs::BREAK_BIT;
		match code {
			Defs::C_SHIFT_L | Defs::C_SHIFT_R => self.keystate.modi.remove(Modifiers::SHIFT),
			Defs::C_ALT => match self.keystate.prefix {
				Prefix::PREFIX1 => self.keystate.modi.remove(Modifiers::ALT_RIGHT),
				_ => self.keystate.modi.remove(Modifiers::ALT_LEFT),
			},
			Defs::C_CTRL => match self.keystate.prefix {
				Prefix::PREFIX1 => self.keystate.modi.remove(Modifiers::CTRL_RIGHT),
				_ => self.keystate.modi.remove(Modifiers::CTRL_LEFT),
			},
			_ => {}
		}
	}

	#[inline(always)]
	pub fn read_status(&self) -> Option<StatusReg> {
		// TODO maybe there is a path which leads to invalid SR, e.g. timeout?
		Some(StatusReg::from_bits_truncate(self.cport.inb()))
	}

	// this should be called by the interrupt handler prologue
	pub fn fetch_key(&mut self) {
		// mask keyboard interrupts when polling.
		let was_masked = Self::is_int_masked();
		if !was_masked {
			Self::disable_keyboard_int();
		}

		// I'd like to see if this panics....
		let sr = self.read_status().unwrap();
		// ignore mouse events
		if !sr.contains(StatusReg::OUTB) || sr.contains(StatusReg::AUXB) {
			return;
		}
		self.update_state(self.dport.inb());
		if !was_masked {
			Self::enable_keyboard_int();
		}
	}

	/// this is safe to be called from all sync levels
	#[inline]
	pub fn consume_key(&mut self) -> Option<Key> {
		let res = self.gather.swap(Key::NONE_KEY, Ordering::Relaxed);
		return Key::from_u32(res);
	}

	pub fn decode_key(&mut self) {
		// the decode_key should not be called when there is no scancode.
		// mask the breakbit
		let s = self.keystate.scan.unwrap();
		let c = s & !Defs::BREAK_BIT;
		let m = self.keystate.modi;
		let p = self.keystate.prefix;

		if c == 53 && p == Prefix::PREFIX1 {
			let k = Key {
				asc: b'/',
				modi: m,
				scan: s,
			};
			self.gather.store(k.to_u32(), Ordering::Relaxed);
			return;
		}

		let asc = if m.contains(Modifiers::NUMLOCK) && p == Prefix::NONE && c >= 71 && c <= 83 {
			ASC_NUM_TAB[c as usize - 71]
		} else if m.contains(Modifiers::ALT_RIGHT) {
			ALT_TAB[c as usize]
		} else if m.contains(Modifiers::SHIFT) {
			SHIFT_TAB[c as usize]
		} else if m.contains(Modifiers::CAPSLOCK) {
			if (c >= 16 && c <= 26) || (c >= 30 && c <= 40) || (c >= 44 && c <= 50) {
				SHIFT_TAB[c as usize]
			} else {
				NORMAL_TAB[c as usize]
			}
		} else {
			NORMAL_TAB[c as usize]
		};

		let k = Key {
			asc,
			modi: m,
			scan: s,
		};

		self.gather.store(k.to_u32(), Ordering::Relaxed);
	}

	pub fn cycle_repeat_rate() {
		todo!();
	}

	pub fn cycle_deley() {
		todo!();
	}

	/// unsafe: this function could block forever as it doesn't emply
	/// timeout.
	/// wait until the next OUTB; return true if get an ACK message
	#[inline(always)]
	unsafe fn __block_for_ack(&self) -> bool {
		loop {
			// if let Some(f)
			let s = self.read_status().unwrap();
			if s.contains(StatusReg::OUTB) {
				break;
			}
		}
		let msg = self.cport.inb();
		return msg == Msg::ACK as u8;
	}

	#[inline(always)]
	unsafe fn __block_until_cmd_buffer_empty(&self) {
		loop {
			let s = self.read_status().unwrap();
			if !s.contains(StatusReg::INB) {
				break;
			};
		}
	}

	/// block until there is something in the data register to read
	#[inline(always)]
	unsafe fn __block_until_data_available(&self) {
		loop {
			let status = self.read_status().unwrap();
			if status.contains(StatusReg::OUTB) {
				break;
			}
		}
	}
}

// x86 and arm has different interrupt controller
#[cfg(target_arch = "x86_64")]
impl KeyboardController {
	#[inline(always)]
	fn disable_keyboard_int() {
		pic_8259::forbid(PD::KEYBOARD);
	}

	#[inline(always)]
	fn enable_keyboard_int() {
		pic_8259::allow(PD::KEYBOARD);
	}

	#[inline(always)]
	fn is_int_masked() -> bool {
		pic_8259::is_masked(PD::KEYBOARD)
	}
}

// for whatever reason the PS/2 keyboard controls the "shutdown"...
// TODO use a "target feature" for machine functions like this.
// TODO "reboot controler" needs better abstraction, maybe a trait.
impl KeyboardController {
	pub unsafe fn reboot(&self) {
		// what ever magic it is, tell BIOS this is an intentional reset and don't
		// run memory test
		println!("reboot...");
		*(0x472 as *mut u16) = 0x1234;
		self.__block_until_cmd_buffer_empty();
		self.cport.outb(Cmd::CpuReset as u8);
	}
}

enum Cmd {
	// these commands are sent through DATA port
	SetLed   = 0xed,
	ScanCode = 0xf0, // Get or set current scancode set
	SetSpeed = 0xf3,
	CpuReset = 0xfe,
}

bitflags! {
#[derive(Debug)]
pub struct StatusReg:u8 {
	const NONE			= 0;
	const OUTB			= 1 << 0;	// output buffer full (can read)
	const INB			= 1 << 1;	// input buffer full (don't write yet)
	const SYS			= 1 << 2;	// System flag, self test success
	const CMD_DATA		= 1 << 3;	// 0: last write to input buffer was data (0x60)
									// 1: last write to input buffer was command (0x64)
	const NOT_LOCKED	= 1 << 4;	// Keyboard Lock. 1: not locked 0: locked
	const AUXB			= 1 << 5;	// AUXB: aux output buffer full.
									// on AT,	1: TIMEOUT
									// on PS/2, 0: keyboard		1: mouse
	const TIMEOUT		= 1 << 6;	// 0: OK 1: Timeout.
	const PARITY_ERR	= 1 << 7;
}
}

enum Msg {
	ACK = 0xfa,
}

pub struct Defs;
impl Defs {
	pub const CTRL: u16 = 0x64;
	pub const DATA: u16 = 0x60;
	pub const CPU_RESET: u8 = 0xfe;
	pub const BREAK_BIT: u8 = 1 << 7;
	// defs of special scan codes.
	pub const C_PREFIX1: u8 = 0xe0;
	pub const C_PREFIX2: u8 = 0xe1;
	pub const C_SHIFT_L: u8 = 0x2a;
	pub const C_SHIFT_R: u8 = 0x36;
	pub const C_ALT: u8 = 0x38;
	pub const C_CTRL: u8 = 0x1d;
	pub const C_CAPSLOCK: u8 = 0x3a;
	pub const C_SCRLOCK: u8 = 0x46;
	pub const C_NUM_P: u8 = 0x45; // NumLock or Pause
	pub const C_F1: u8 = 0x3b;
	pub const C_DEL: u8 = 0x53;
	pub const C_UP: u8 = 0x48;
	pub const C_DOWN: u8 = 0x50;
	pub const C_LEFT: u8 = 0x4b;
	pub const C_RIGHT: u8 = 0x4d;
	pub const C_DIV: u8 = 0x8;
}
