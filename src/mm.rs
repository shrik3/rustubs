//! memory management unit

mod pma;
pub mod vmm;

use crate::arch::x86_64::paging::{get_root, Pagetable};
use crate::defs::*;
use crate::machine::multiboot;
use alloc::alloc::{alloc, alloc_zeroed, dealloc, Layout};
use alloc::vec::Vec;
use core::arch::asm;
use core::ops::Range;
use lazy_static::lazy_static;
use linked_list_allocator::LockedHeap;
use spin::Mutex;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

lazy_static! {
	pub static ref KSTACK_ALLOCATOR: Mutex<KStackAllocator> =
		Mutex::new(KStackAllocator::new());
}

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
		if mblock.get_end() <= ExternSyms::___KERNEL_PM_START__ as u64 {
			continue;
		}
		let mut r = mblock.get_range();
		if r.contains(&(ExternSyms::___KERNEL_PM_END__ as u64)) {
			assert!(
				r.contains(&(ExternSyms::___KERNEL_PM_START__ as u64)),
				"FATAL: kernel physical map cross physical blocks, how?"
			);
			r.start = ExternSyms::___KERNEL_PM_END__ as u64;
		}
		// TODO this is pretty ugly...
		match largest_phy_range {
			None => largest_phy_range = Some(r),
			Some(ref lr) => {
				if (r.end - r.start) > (lr.end - lr.start) {
					largest_phy_range = Some(r);
				}
			}
		}
	}

	let pr = &largest_phy_range.expect("Can't find usable physical block");
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

/// wrapper around the global allocator with caching
pub struct KStackAllocator {
	pool: Vec<u64>,
}

/// TODO: the heap allocator is primitive atm and it may fail to allocate new
/// kernel stack (64K here) due to fragmentation. It may be a good idea to
/// reserve some memory during system init to guarantee that we can at least
impl KStackAllocator {
	const KSTACK_ALLOC_POOL_CAP: usize = 16;
	const KSTACK_LAYOUT: Layout = unsafe {
		Layout::from_size_align_unchecked(
			Mem::KERNEL_STACK_SIZE as usize,
			Mem::KERNEL_STACK_SIZE as usize,
		)
	};

	pub fn new() -> Self {
		let p = Vec::with_capacity(Self::KSTACK_ALLOC_POOL_CAP);
		Self { pool: p }
	}

	/// unsafe because this may fail (same as populate)
	pub unsafe fn allocate(&mut self) -> u64 {
		if let Some(addr) = self.pool.pop() {
			return addr;
		} else {
			return alloc(Self::KSTACK_LAYOUT) as u64;
		}
	}

	/// unsafe because you must make sure you give back something the allocator gave
	/// you. Otherwise you break the kernel heap allocator.
	pub unsafe fn free(&mut self, addr: u64) {
		if self.pool.len() < Self::KSTACK_ALLOC_POOL_CAP {
			self.pool.push(addr);
		} else {
			dealloc(addr as *mut u8, Self::KSTACK_LAYOUT);
		}
	}

	/// unsafe because this could OOM if you stress the allocator too much
	/// (although unlikely)
	pub unsafe fn populate(&mut self) {
		for _ in 0..Self::KSTACK_ALLOC_POOL_CAP {
			self.pool.push(alloc(Self::KSTACK_LAYOUT) as u64);
		}
	}
}

const LAYOUT_4K_ALIGNED: Layout =
	unsafe { Layout::from_size_align_unchecked(0x1000, 0x1000) };
/// allocate 4k aligned memory.
/// TODO create a buffer (like in KStackAllocator) for performance.
pub fn allocate_4k() -> u64 {
	return unsafe { alloc(LAYOUT_4K_ALIGNED) } as u64;
}
pub fn allocate_4k_zeroed() -> u64 {
	return unsafe { alloc_zeroed(LAYOUT_4K_ALIGNED) } as u64;
}

/// invalidate a single page mapping in tlb
pub fn invlpg(va: u64) { unsafe { asm!("invlpg [{0}]", in(reg) va) }; }

/// flush the whole tlb
pub fn flush_tlb() {
	unsafe {
		asm!(
			"
		push rax;
		mov rax, cr3;
		mov cr3, rax;
		pop rax;
		"
		)
	}
}

/// drop the low memory mapping from the current pagetable by removing the first
/// entry from pml4 table (which mapps to 0~512G). The PDP table is unchanged,
/// wasting 4K of memory but there is nothing we can do now since the heap
/// allocator doesn't manage this address.
///
/// after calling this function, the system can no longer directly access memory
/// by physical address
pub unsafe fn drop_init_mapping() {
	let pt: &mut Pagetable = unsafe { &mut *(get_root() as *mut Pagetable) };
	pt.entries[0].set_unused();
	flush_tlb();
}
