use bitflags::bitflags;
use core::convert;
use core::ffi::c_uchar;

#[derive(Copy, Clone, Debug)]
pub struct Key {
	pub asc: c_uchar,
	pub scan: u8,
	pub modi: Modifiers,
}

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
	#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
			asc: 0,  // logically scan + modi.shift => asc
			scan: 0, // scancode, "raw"
			modi: Modifiers::NONE,
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

// scan codes of a few specific keys
pub enum Scan {
	F1 = 0x3b,
	Del = 0x53,
	Up = 72,
	Down = 80,
	Left = 75,
	Right = 77,
	Div = 8,
}

// Decoding tables ... this shit is so ugly, thanks to rust's strong typing system!!!
// Also, this is a german layout keyboard
// oh btw, the code translation is done by ChatGPT if it's wrong complain to the AI!
pub const NORMAL_TAB: [c_uchar; 89] = [
	0, 0, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 225, 39, 8, 0, 113, 119, 101, 114, 116, 122, 117,
	105, 111, 112, 129, 43, 10, 0, 97, 115, 100, 102, 103, 104, 106, 107, 108, 148, 132, 94, 0, 35,
	121, 120, 99, 118, 98, 110, 109, 44, 46, 45, 0, 42, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 45, 0, 0, 0, 43, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0,
];

pub const SHIFT_TAB: [c_uchar; 89] = [
	0, 0, 33, 34, 21, 36, 37, 38, 47, 40, 41, 61, 63, 96, 0, 0, 81, 87, 69, 82, 84, 90, 85, 73, 79,
	80, 154, 42, 0, 0, 65, 83, 68, 70, 71, 72, 74, 75, 76, 153, 142, 248, 0, 39, 89, 88, 67, 86,
	66, 78, 77, 59, 58, 95, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0, 0,
];

pub const ALT_TAB: [c_uchar; 89] = [
	0, 0, 0, 253, 0, 0, 0, 0, 123, 91, 93, 125, 92, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 126,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 230, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124, 0, 0,
];

pub const ASC_NUM_TAB: [c_uchar; 13] = [55, 56, 57, 45, 52, 53, 54, 43, 49, 50, 51, 48, 44];
pub const SCAN_NUM_TAB: [c_uchar; 13] = [8, 9, 10, 53, 5, 6, 7, 27, 2, 3, 4, 11, 51];
