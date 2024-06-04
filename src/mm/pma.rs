use crate::defs::*;
use crate::io::*;
use crate::machine::multiboot::MultibootMmap;
use core::ops::Range;
use core::slice;

extern "C" {
	fn ___FREE_PAGE_STACK__();
}

/// There should only be one global instance of this.
pub struct PageStackAllocator {
	page_stack: &'static mut [u64],
	size: usize,
	head: usize,
}

impl PageStackAllocator {
	// covering 4GiB physical memory of 4K frames
	const STACK_SIZE: usize = 0x100000;

	pub fn new() -> Self {
		let ps = Self {
			page_stack: unsafe {
				slice::from_raw_parts_mut(
					P2V(___FREE_PAGE_STACK__ as u64).unwrap() as *mut u64,
					Self::STACK_SIZE,
				)
			},
			size: Self::STACK_SIZE,
			head: 0,
		};
		return ps;
	}

	/// push an addr into the free page stack
	/// MUST be atomic or bad things happen...
	pub fn free_page(&mut self, addr: u64) -> bool {
		if self.head >= self.size {
			return false;
		}
		self.page_stack[self.head] = addr;
		self.head += 1;
		return true;
	}

	pub fn alloc_page(&mut self) -> Option<u64> {
		if self.head == 0 {
			return None;
		}
		self.head -= 1;
		Some(self.page_stack[self.head])
	}

	/// 4k page only?
	pub fn insert_range(&mut self, r: &Range<u64>) -> u64 {
		// r.contains(&1);
		let mut inserted = 0;
		let mut page = roundup_4k(r.start);
		loop {
			if !r.contains(&page) {
				break;
			}
			if !self.free_page(page) {
				break;
			} else {
				inserted += 1;
			}
			page += 0x1000;
		}
		return inserted;
	}
}
