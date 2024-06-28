pub mod fault;
pub mod pagetable;
use crate::defs;
use crate::defs::rounddown_4k;
use crate::defs::P2V;
use crate::io::*;
use crate::mm::allocate_4k_zeroed;
use crate::mm::vmm::VMArea;
use crate::mm::vmm::VMType;
use core::arch::asm;
use core::ops::Range;
use core::ptr;
pub use pagetable::*;
/// for x86_64, return the CR3 register. this is the **physical** address of the
/// page table root.
/// TODO: use page root in task struct instead of raw cr3
#[inline]
pub fn get_cr3() -> u64 {
	let cr3: u64;
	unsafe { asm!("mov {}, cr3", out(reg) cr3) };
	cr3
}

/// returns the identically mapped (+ kernel offset) virtual address of the page
/// table
#[inline]
pub fn get_root() -> u64 { P2V(get_cr3()).unwrap() }

/// unsafe as it dereferences raw pointer pt_root. Must make sure it's a valid,
/// 4k aligned _virtual_ address.
// TODO use Result type instead of bool so that we can do early return with ?..
pub unsafe fn map_vma(pt_root: u64, vma: &VMArea, do_copy: bool) -> bool {
	// create mappings in pagetable
	let flags = PTEFlags::PRESENT | PTEFlags::WRITABLE | PTEFlags::USER;
	if !map_range(pt_root, &vma.vm_range, flags) {
		println!("failed to map range");
		return false;
	}
	if !do_copy {
		return true;
	}
	match vma.backing {
		VMType::ANOM => {
			return true;
		}
		VMType::FILE(f) => {
			if !do_copy {
				return true;
			}
			let sz = (vma.vm_range.end - vma.vm_range.start) as usize;
			sprintln!("copy from {:p} to {:#X}", &f[0], vma.vm_range.start);
			unsafe {
				ptr::copy_nonoverlapping(
					&f[0] as *const u8,
					vma.vm_range.start as *mut u8,
					sz,
				)
			}
			return true;
		}
		_ => {
			println!("unknown backing");
			return false;
		}
	}
}

pub fn map_range(pt_root: u64, r: &Range<u64>, flags: PTEFlags) -> bool {
	let mut va_aligned = rounddown_4k(r.start);
	while va_aligned < r.end {
		if !map_page(pt_root, va_aligned, flags) {
			println!("failed to map page @ {:#X}", va_aligned);
			return false;
		}
		va_aligned += defs::Mem::PAGE_SIZE;
	}
	return true;
}

/// walk the page table, create missing tables, return mapped physical frame
pub fn map_page(pt_root: u64, va: u64, _flags: PTEFlags) -> bool {
	let pt = pt_root as *mut Pagetable;
	if !defs::is_aligned_4k(va) {
		println!("not aligned");
		return false;
	}
	let flags: u64 = _flags.bits();
	let l4idx = pagetable::p4idx(va) as usize;
	let l3idx = pagetable::p3idx(va) as usize;
	let l2idx = pagetable::p2idx(va) as usize;
	let l1idx = pagetable::p1idx(va) as usize;
	let mut require_new = false;
	unsafe {
		let l4_ent = &mut (*pt).entries[l4idx];
		let l3_tbl: *mut Pagetable;
		if l4_ent.is_unused() || require_new {
			l3_tbl = allocate_4k_zeroed() as *mut Pagetable;
			l4_ent.entry = defs::V2P(l3_tbl as u64).unwrap() | flags;
			require_new = true
		} else {
			l3_tbl = defs::P2V(l4_ent.addr()).unwrap() as *mut Pagetable;
		}
		let l3_ent = &mut (*l3_tbl).entries[l3idx];
		let l2_tbl: *mut Pagetable;
		if l3_ent.is_unused() || require_new {
			l2_tbl = allocate_4k_zeroed() as *mut Pagetable;
			l3_ent.entry = defs::V2P(l2_tbl as u64).unwrap() | flags;
			require_new = true
		} else {
			l2_tbl = defs::P2V(l3_ent.addr()).unwrap() as *mut Pagetable;
		}
		let l2_ent = &mut (*l2_tbl).entries[l2idx];
		let l1_tbl: *mut Pagetable;
		if l2_ent.is_unused() || require_new {
			l1_tbl = allocate_4k_zeroed() as *mut Pagetable;
			l2_ent.entry = defs::V2P(l1_tbl as u64).unwrap() | flags;
			require_new = true
		} else {
			l1_tbl = defs::P2V(l2_ent.addr()).unwrap() as *mut Pagetable;
		}
		let pte = &mut (*l1_tbl).entries[l1idx];
		if pte.is_unused() || require_new {
			let page = allocate_4k_zeroed();
			pte.entry = defs::V2P(page).unwrap() | flags;
		} else {
			// TODO we need to free this frame
			panic!("PTE already taken: {:#X}", pte.entry);
		}
		// flush tlb
		asm!("invlpg [{0}]", in(reg) va);
	}
	return true;
}
