//! (deprecated) a simple stack based 4K physical frame allocator.
//!
//! Adcantages
//! 1. allocation and deallocation is O(1).
//! 2. can manage fragmented (non-consecutive) memory regions and doesn't suffer
//!    fragmentation because memory is fully virtualized.
//!
//! Limitations
//! 1. slow initialization: needs to traverse all available pages and push their
//!    address one by one
//! 2. can't give continous physical memory of more than 4K
//! 3. the allocator stack itself requires continous space (it's an array), and
//!    has a higher storage overhead: every page requires 8 bytes for the
//!    address, that's `8/4096 == 1/512`.
//! 4. (kinda?) conflicts kmalloc/vmalloc design in e.g. linux kernel, kmalloc
//!    manages the __identically mapped__ virtual memory in the kernel address
//!    space, that is `0xffff800000000000 + 64G` in rustubs, in another word,
//!    the kmalloc manages the physical memory: obviously this one only gives 4K
//!    memory and can't work as a kmalloc backend!

use crate::defs::*;
use core::ops::Range;
use core::slice;
// disabled for now
// lazy_static! {
// 	pub static ref GLOBAL_PMA: Mutex<pma::PageStackAllocator> =
// 		Mutex::new(pma::PageStackAllocator::new());
// }

/// There should only be one global instance of this.
pub struct PageStackAllocator {
	page_stack: &'static mut [u64],
	size: usize,
	head: usize,
}

#[allow(dead_code)]
impl PageStackAllocator {
	// covering 4GiB physical memory of 4K frames
	const STACK_SIZE: usize = 0x100000;

	pub fn new() -> Self {
		let ps = Self {
			page_stack: unsafe {
				slice::from_raw_parts_mut(
					P2V(ExternSyms::___FREE_PAGE_STACK__ as u64).unwrap()
						as *mut u64,
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
