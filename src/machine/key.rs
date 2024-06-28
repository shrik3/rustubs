use bitflags::bitflags;
use core::ffi::c_uchar;
use core::mem::transmute;

#[derive(Copy, Clone, Debug)]
#[repr(C, align(4))]
/// packed into 32bit C struct so that we can use AtomicU32 for L2/L3
/// synchronization. We mask the highest byte to indicate "None"
pub struct Key {
	pub asc: c_uchar,
	pub scan: u8,
	pub modi: Modifiers,
}

impl Key {
	pub const NONE_KEY: u32 = 0xff00_0000;

	pub fn to_u32(&self) -> u32 { unsafe { transmute::<Key, u32>(*self) } }

	pub fn from_u32(k: u32) -> Option<Self> {
		if Self::is_none(k) {
			None
		} else {
			Some(unsafe { transmute::<u32, Self>(k) })
		}
	}

	pub fn is_none(k: u32) -> bool { k & Self::NONE_KEY != 0 }
}

bitflags! {
	/// Technically the *Lock are special keys, instead of Modifiers
	/// but we don't need another type FWIW.
	/// Mask `bits[2:0]` to get the leds.
	#[derive(Copy, Clone, Eq, PartialEq, Debug)]
	pub struct Modifiers:u8 {
		const NONE			= 0;
		// lock states
		const SCROLL_LOCK	= 1 << 0;
		const NUMLOCK		= 1 << 1;
		const CAPSLOCK		= 1 << 2;
		// modifiers
		const SHIFT			= 1 << 3;
		const ALT_LEFT		= 1 << 4;
		const ALT_RIGHT		= 1 << 5;
		const CTRL_LEFT		= 1 << 6;
		const CTRL_RIGHT	= 1 << 7;
	}
}

impl Key {
	pub fn new() -> Self {
		Self {
			asc: 0,  // logically scan + modi.shift => asc
			scan: 0, // scancode, "raw"
			modi: Modifiers::NONE,
		}
	}
}

/// scan codes of a few specific keys
pub enum Scan {
	F1    = 0x3b,
	Del   = 0x53,
	Up    = 72,
	Down  = 80,
	Left  = 75,
	Right = 77,
	Div   = 8,
}

// oh btw, the code translation is done by ChatGPT if it's wrong complain to the AI!
#[rustfmt::skip]
pub const NORMAL_TAB: [c_uchar; 100] = [
	0  , 0  , 49 , 50 , 51 , 52 , 53 , 54 , 55 , 56 ,
	57 , 48 , 45 , 61 , 8  , 0  , 113, 119, 101, 114,
	116, 121, 117, 105, 111, 112, 91 , 93 , 10 , 0  ,
	97 , 115, 100, 102, 103, 104, 106, 107, 108, 59 ,
	39 , 96 , 0  , 134, 122, 120, 99 , 118, 98,  110,
	109, 44 , 46 , 47 , 0  , 42 , 0  , 32 , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 45 , 0  , 0  , 0  , 43 , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 60 , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
];

#[rustfmt::skip]
pub const SHIFT_TAB: [c_uchar; 100] = [
	0  , 0  , 33 , 64 , 35 , 36 , 37 , 94 , 47 , 42 ,
	40 , 41 , 95 , 43 , 0  , 0  , 81 , 87 , 69 , 82 ,
	84 , 89 , 85 , 73 , 79 , 80 , 123, 125 , 0  , 0  ,
	65 , 83 , 68 , 70 , 71 , 72 , 74 , 75 , 76 , 58,
	34 , 126, 0  , 124, 90 , 88 , 67 , 86 , 66 , 78 ,
	77 , 60 , 62 , 63 , 0  , 0  , 0  , 32 , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 62 , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
];

#[rustfmt::skip]
pub const ALT_TAB: [c_uchar; 100] = [
	0  , 0  , 0  , 253, 0  , 0  , 0  , 0  , 123, 91 ,
	93 , 125, 92 , 0  , 0  , 0  , 64 , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 126, 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	230, 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 124, 0  , 0  , 0  ,
	0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  , 0  ,
];
#[rustfmt::skip]
pub const ASC_NUM_TAB: [c_uchar; 13] = [55, 56, 57, 45, 52, 53, 54, 43, 49, 50, 51, 48, 44];
#[rustfmt::skip]
pub const SCAN_NUM_TAB: [c_uchar; 13] = [8, 9, 10, 53, 5, 6, 7, 27, 2, 3, 4, 11, 51];
