use self::super::kbd_defs::*;
use core::convert;

pub struct Key {
	asc: u8,
	scan: u8,
	modi: u8,
	rawcode: u8, // this field not necessary, remove after testing
}

// Not implementing
// +operator char()
//

impl convert::Into<char> for Key {
	fn into(self) -> char {
		self.asc as char
	}
}

impl convert::Into<u8> for Key {
	fn into(self) -> u8 {
		self.asc
	}
}

#[allow(dead_code)]
impl Key {
	pub fn new() -> Self {
		Self {
			asc: 0,
			scan: 0,
			modi: 0,
			rawcode: 0,
		}
	}

	pub fn valid(self) -> bool {
		self.scan != 0
	}

	pub fn invalidate(&mut self) {
		self.scan = 0;
	}

	pub fn set_raw(&mut self, code: u8) {
		self.rawcode = code;
	}

	pub fn get_raw(self) -> u8 {
		self.rawcode
	}

	// setter and getter for ascii and scancode
	pub fn set_ascii(&mut self, ascii: u8) {
		self.asc = ascii;
	}
	pub fn get_ascii(self) -> u8 {
		self.asc
	}
	pub fn set_scancode(&mut self, scancode: u8) {
		self.scan = scancode;
	}
	pub fn get_scancode(self) -> u8 {
		self.scan
	}

	// reading the state of SHIFT, ALT, CTRL etc.
	pub fn shift(&self) -> bool {
		self.modi & (Mbit::Shift as u8) != 0
	}
	pub fn alt_left(&self) -> bool {
		self.modi & (Mbit::AltLeft as u8) != 0
	}
	pub fn alt_right(&self) -> bool {
		self.modi & (Mbit::AltRight as u8) != 0
	}
	pub fn ctrl_left(&self) -> bool {
		self.modi & (Mbit::CtrlLeft as u8) != 0
	}
	pub fn ctrl_right(&self) -> bool {
		self.modi & (Mbit::CtrlRight as u8) != 0
	}
	pub fn caps_lock(&self) -> bool {
		self.modi & (Mbit::CapsLock as u8) != 0
	}
	pub fn num_lock(&self) -> bool {
		self.modi & (Mbit::NumLock as u8) != 0
	}
	pub fn scroll_lock(&self) -> bool {
		self.modi & (Mbit::ScrollLock as u8) != 0
	}
	pub fn alt(&self) -> bool {
		self.alt_left() | self.alt_right()
	}
	pub fn ctrl(&self) -> bool {
		self.ctrl_left() | self.ctrl_right()
	}

	// setting/clearing states of SHIFT, ALT, CTRL etc.
	pub fn set_shift(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::Shift as u8
		} else {
			self.modi & !(Mbit::Shift as u8)
		}
	}

	pub fn set_alt_left(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::AltLeft as u8
		} else {
			self.modi & !(Mbit::AltLeft as u8)
		}
	}

	pub fn set_alt_right(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::AltRight as u8
		} else {
			self.modi & !(Mbit::AltRight as u8)
		}
	}

	pub fn set_ctrl_left(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::CtrlLeft as u8
		} else {
			self.modi & !(Mbit::CtrlLeft as u8)
		}
	}

	pub fn set_ctrl_right(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::CtrlRight as u8
		} else {
			self.modi & !(Mbit::CtrlRight as u8)
		}
	}

	pub fn set_caps_lock(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::CapsLock as u8
		} else {
			self.modi & !(Mbit::CapsLock as u8)
		}
	}

	pub fn set_num_lock(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::NumLock as u8
		} else {
			self.modi & !(Mbit::NumLock as u8)
		}
	}

	pub fn set_scroll_lock(&mut self, pressed: bool) {
		self.modi = if pressed {
			self.modi | Mbit::ScrollLock as u8
		} else {
			self.modi & !(Mbit::ScrollLock as u8)
		}
	}
}
