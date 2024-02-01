use self::super::key::*;
use crate::machine::device_io::*;
use bitflags::bitflags;
use core::cmp;
use core::cmp::{Eq, PartialEq};
use num_enum::{IntoPrimitive, TryFromPrimitive};

// Driver for the PS/2 keybard/mouse controller
// beyound OOStuBS: I'm gonna write full driver for it.

// TODO figure out how to abstract the "interrupt control" layer.
// Keyboard controller should not be arch dependent
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::interrupt::{pic_8259, pic_8259::PicDeviceInt as PD};

// this is the driver for keyboard controller not to confuse with the keyboard module.
// The later is an abstraction
// This one serves a the HW driver

// TODO
// [functions]
// Keyboard_Controller()
// get_ascii_code()
// key_decoded()
// key_hit()
// reboot()
// set_led(char led,bool on)
// set_repeat_rate(int speed,int delay)

pub struct KeyboardController {
	code: u8,
	prefix: u8,
	leds: Led,
	current: Option<Key>,
	last: Option<Key>,
	gather: Option<Key>,
	cport: IOPort,
	dport: IOPort,
}

pub enum KeyDelay {
	L0 = 0,
	L1 = 1,
	L2 = 2,
	L3 = 3,
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
	fn toggle_keyboard_int(enable: bool) {
		if enable {
			Self::enable_keyboard_int()
		} else {
			Self::disable_keyboard_int()
		}
	}

	#[inline(always)]
	fn is_int_masked() -> bool {
		pic_8259::is_masked(PD::KEYBOARD)
	}
}

impl KeyboardController {
	pub fn new() -> Self {
		Self {
			code: 0,
			prefix: 0,
			leds: Led::NONE,
			last: None,
			current: None,
			gather: None,
			cport: IOPort::new(Defs::CTRL),
			dport: IOPort::new(Defs::DATA),
		}
	}

	pub fn fetch_key(&mut self) -> Key {
		todo!();
		// this should be called by the interrupt handler.
		// 1. read raw keycode from data port
		// 2. try to decode and put the result into self.gather.
		// TODO consider move the decoding into the epilogue.
	}

	// key decoding is stateful, this can't be implemented into Key struct.
	pub fn decode_key(&mut self) -> bool {
		// try to decode the raw 'code' into self.gather
		// return true upon success
		// this is a transcription of the C code.... Improve later.
		// OOStuBS: The keys that were added in MF II keyboards -- compared to the older AT
		//			keyboard -- always send one of two possible prefix bytes first.
		let done = false;
		if self.code == Defs::PREFIX1 || self.code == Defs::PREFIX2 {
			self.prefix = self.code;
			return false;
		}

		// OOStuBS: Releasing a key is actually only interesting in this implementation
		//			for the "modifier" keys SHIFT, CTRL and ALT.  For the other keys, we
		//			can ignore the break code.
		//			; A Key's break code is identical to its make code with break_bit set

		false
	}

	// block until status.outb becomes 1
	// return false on invalid SR.
	// these wait_ functions are unsafe. If used inproperly this becomes a deadloop
	// TODO support software timeout
	unsafe fn wait_write(&self) {
		loop {
			let sr = StatusReg::from_bits_truncate(self.cport.inb());
			if sr.contains(StatusReg::OUTB) {
				break;
			}
		}
	}

	// block until status.inb becomes 0
	unsafe fn wait_read(&self) {
		loop {
			let sr = StatusReg::from_bits_truncate(self.cport.inb());
			if !sr.contains(StatusReg::INB) {
				break;
			}
		}
	}

	pub fn set_repeat_rate_delay(rate: u8, delay: KeyDelay) {
		let rate = cmp::min(rate, 31);
		let delay = delay as u8;
		let is_masked = Self::is_int_masked();
		// idsable keyboard interrupt
		// TODO should have a timeout
		Self::disable_keyboard_int();

		Self::toggle_keyboard_int(is_masked);
	}

	pub fn reboot(&mut self) {
		todo!();
	}
}

// I think constants are more handy than enum for these...
// Keyboard controller commands

enum Cmd {
	// these commands are sent through DATA port
	SetLed = 0xed,
	ScanCode = 0xf0, // Get or set current scancode set
	SetSpeed = 0xf3,
}

bitflags! {
pub struct Led:u8 {
	const NONE			= 0;
	const SCROLL_LOCK	= 1<<0;
	const NUM_LOCk		= 1<<1;
	const CAPS_LOCK		= 1<<2;
}
}

bitflags! {
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
	pub const BREAK_BIT: u8 = 0x80;
	pub const PREFIX1: u8 = 0xe0;
	pub const PREFIX2: u8 = 0xe1;
}
