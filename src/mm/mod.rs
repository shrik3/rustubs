use crate::defs::*;
use crate::io::*;
use crate::machine::multiboot;
use core::ops::Range;
use linked_list_allocator::LockedHeap;

pub mod pma;

use lazy_static::lazy_static;
use spin::Mutex;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// half measure: simply initialize the linkedlist allocator
pub fn init() {
	let mbi = multiboot::get_mb_info().unwrap();
	let mmapinfo = unsafe { mbi.get_mmap() }.unwrap();
	let buf_start = mmapinfo.mmap_addr;
	let buf_len = mmapinfo.mmap_length;
	let buf_end = buf_start + buf_len;
	let mut curr = buf_start as u64;
	// initialize the heap allocator with the largest physical memory block
	let mut largest_phy_range: Option<Range<u64>> = None;
	loop {
		if curr >= buf_end as u64 {
			break;
		}
		let mblock = unsafe { &*(curr as *const multiboot::MultibootMmap) };
		curr += mblock.size as u64;
		curr += 4;
		if mblock.mtype != multiboot::MultibootMmap::MTYPE_RAM {
			continue;
		}
		if mblock.get_end() <= unsafe { pmap_kernel_start() } {
			continue;
		}
		// TODO early break if the array is already full
		let mut r = mblock.get_range();
		if r.contains(&unsafe { pmap_kernel_end() }) {
			assert!(
				r.contains(&unsafe { pmap_kernel_start() }),
				"FATAL: kernel physical map cross physical blocks, how?"
			);
			r.start = unsafe { pmap_kernel_end() };
		}
		match largest_phy_range {
			None => largest_phy_range = Some(r),
			Some(ref lr) => {
				if (r.end - r.start) > (lr.end - lr.start) {
					largest_phy_range = Some(r);
				}
			}
		}
	}

	let pr = &largest_phy_range.expect("Can't find any available physical block");
	assert!((pr.end - pr.start) >= Mem::MIN_PHY_MEM, "TO LITTLE RAM ...");
	// init heap allocator on id map
	unsafe {
		ALLOCATOR.lock().init(
			P2V(pr.start).unwrap() as *mut u8,
			(pr.end - pr.start) as usize,
		);
	}
	println!(
		"[init] mm: heap alloc initialized @ {:#X} - {:#X}",
		P2V(pr.start).unwrap(),
		P2V(pr.end).unwrap()
	);
}

/// populate the physical frame pool. This conflicts with the kernel heap allocator (kmalloc),
/// which operates on the id map regions.
pub fn _init_pma() {
	todo!()
}
