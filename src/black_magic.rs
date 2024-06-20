//! collection of hacks
use core::slice;

pub unsafe fn make_static<'a>(r: &'a [u8]) -> &'static [u8] {
	return slice::from_raw_parts(&r[0] as *const u8, r.len());
}
