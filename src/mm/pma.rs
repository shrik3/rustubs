use crate::defs::Mem;
use core::ffi::c_void;
use core::{ptr, slice};
// this is POC code, it will be ugly

extern "C" {
	pub fn ___KERNEL_END__();
}
/// Bitmap for physical frames. to get around the chicken-egg problem, we provide a provisional
/// bitmap of fixed length in the startup code.
pub struct PFMap {
	bm: &'static mut [u8],
	skip: usize, // pfn to skip (because they are already used by the initial kernel image)
	end: usize,  // pfn limit
}

// TODO PMA initialization : needs to singleton

impl PFMap {}

pub struct Frame {
	pfn: usize,
}

impl Frame {
	pub fn addr(&self) -> usize {
		self.pfn << Mem::PAGE_SHIFT
	}
}

// pub struct PageAlloctor;
