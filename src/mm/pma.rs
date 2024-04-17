use crate::defs::{self, Mem};
use core::ffi::c_void;
use core::{ptr, slice};
// this is POC code, it will be ugly

extern "C" {
	pub fn ___KERNEL_END__();
}

type BitU8 = u8;
/// Bitmap for physical frames
pub struct FMap {
	pub bm: &'static mut [BitU8],
	// skip over the kernel image and the bitmap itself.
	pub skip_byte: usize,
}

pub enum PMAError {
	DoubleFree,
}

impl FMap {
	pub fn new() -> Self {
		let map_start = ___KERNEL_END__ as usize;
		let fmap = Self {
			bm: unsafe { slice::from_raw_parts_mut(map_start as *mut u8, Mem::PHY_BM_SIZE) },
			// looks ugly, perhaps FIXME
			// We'll waste several frames for the sake of easy alignment
			skip_byte: 1 + ((map_start >> Mem::PAGE_SHIFT) / 8),
		};
		fmap
	}

	/// return : index to the bitmap u8 , bit mask to retrive the bit.
	fn locate_bit(addr: usize) -> Option<(usize, u8)> {
		if addr >= Mem::PHY_TOP {
			return None;
		}
		let pn = addr >> Mem::PAGE_SHIFT;
		let idx = pn / 8;
		let mask: u8 = 1 << (pn % 8);
		Some((idx, mask))
	}

	pub fn alloc_frame(&mut self) -> usize {
		for i in self.skip_byte..self.bm.len() {
			if self.bm[i] == 0xff {
				continue;
			}
			todo!()
		}
		0
	}

	pub fn dealloc_frame(&mut self) -> Result<(), PMAError> {
		Ok(())
	}

	pub fn init(&mut self) {
		for i in 0..self.skip_byte {
			self.bm[i] = 0xff;
		}
		for i in self.skip_byte..self.bm.len() {
			self.bm[i] = 0;
		}
	}
}
