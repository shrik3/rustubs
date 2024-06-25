//! collection of hacks
use core::slice;

pub unsafe fn make_static(r: &[u8]) -> &'static [u8] {
	return slice::from_raw_parts(&r[0] as *const u8, r.len());
}

// an empty struct
pub struct Empty {}
impl Empty {
	pub const fn new() -> Self {
		Self {}
	}
}
