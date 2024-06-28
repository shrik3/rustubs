//! collection of hacks
use core::slice;

pub unsafe fn make_static(r: &[u8]) -> &'static [u8] {
	return slice::from_raw_parts(&r[0] as *const u8, r.len());
}

// an empty struct
pub struct Empty {}
impl Empty {
	pub const fn new() -> Self { Self {} }
}

/// flush a volatile variable: rust doesn't have a volatile keyword. When a
/// "const static" variable is expected to be written externally the optimized
/// code may go wrong.
pub fn flush<T>(thing: &T) -> T {
	unsafe { core::ptr::read_volatile(thing as *const T) }
}
