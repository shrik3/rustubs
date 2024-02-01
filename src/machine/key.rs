use self::super::kbd_defs::*;
use bitflags::bitflags;
use core::convert;

pub struct Key {
	asc: u8,
	scan: u8,
	modi: Modifiers,
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

bitflags! {
	pub struct Modifiers:u8 {
		const NONE			= 0;
		const SHIFT			= 1 << 0;
		const ALT_LEFT		= 1 << 1;
		const ALT_RIGHT		= 1 << 2;
		const CTRL_LEFT		= 1 << 3;
		const CTRL_RIGHT	= 1 << 4;
		const CAPSLOCK		= 1 << 5;
		const NUMLOCK		= 1 << 6;
		const SCROLL_LOCK	= 1 << 7;
	}
}

#[allow(dead_code)]
impl Key {
	pub fn new() -> Self {
		Self {
			asc: 0,
			scan: 0,
			modi: Modifiers::NONE,
			rawcode: 0,
		}
	}
	pub fn decode(&mut self) {
		// decode  key
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

	// TODO the setters and getters should not be their own functions....

	#[inline(always)]
	pub fn mod_contains(&self, modi: Modifiers) -> bool {
		self.modi.contains(modi)
	}

	#[inline(always)]
	pub fn mod_set(&mut self, modi: Modifiers, pressed: bool) {
		if pressed {
			self.modi.insert(modi);
		} else {
			self.modi.remove(modi);
		}
	}
}
