// code derived from the x86_64 crate
// https://docs.rs/x86_64/latest/src/x86_64/addr.rs.html
// see ATTRIBUTIONS
pub mod fault;
use bitflags::bitflags;
use core::arch::asm;

use crate::P2V;
#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct Pagetable {
	pub entries: [PTE; Self::ENTRY_COUNT],
}

#[derive(Clone)]
#[repr(transparent)]
pub struct PTE {
	entry: u64,
}

// use wrapped VA and PA instead of simply u64 as a sanity check:
// VA must be sign extended, PA has at most 52 bits
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct VAddr(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PAddr(u64);

bitflags! {
#[derive(Debug)]
pub struct PTEFlags:u64 {
	const ZERO      = 0;
	const PRESENT   = 1 << 0;
	const WRITABLE  = 1 << 1;
	const USER      = 1 << 2;
	const WT        = 1 << 3;
	const NC        = 1 << 4;
	const ACCESSED  = 1 << 5;
	const DIRTY     = 1 << 6;
	const HUGE_PAGE = 1 << 7;
	const GLOBAL    = 1 << 8;
	const B9        = 1 << 9;
	const B10       = 1 << 10;
	const B11       = 1 << 11;
	// [51:12] is used for translation address
	// [62:52] are user defined.
	// [63] NO_EXECUTE, needs to be enabled in EFER.
	const NE        = 1 << 63;
}
}

impl Pagetable {
	const ENTRY_COUNT: usize = 512;
	/// Creates an empty page table.
	#[inline]
	pub const fn new() -> Self {
		const EMPTY: PTE = PTE::new();
		Pagetable {
			entries: [EMPTY; Self::ENTRY_COUNT],
		}
	}

	/// Clears all entries.
	#[inline]
	pub fn zero(&mut self) {
		for entry in self.iter_mut() {
			entry.set_unused();
		}
	}

	/// Returns an iterator over the entries of the page table.
	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &PTE> {
		(0..512).map(move |i| &self.entries[i])
	}

	/// Returns an iterator that allows modifying the entries of the page table.
	#[inline]
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PTE> {
		// Note that we intentionally don't just return `self.entries.iter()`:
		// Some users may choose to create a reference to a page table at
		// `0xffff_ffff_ffff_f000`. This causes problems because calculating
		// the end pointer of the page tables causes an overflow. Therefore
		// creating page tables at that address is unsound and must be avoided.
		// Unfortunately creating such page tables is quite common when
		// recursive page tables are used, so we try to avoid calculating the
		// end pointer if possible. `core::slice::Iter` calculates the end
		// pointer to determine when it should stop yielding elements. Because
		// we want to avoid calculating the end pointer, we don't use
		// `core::slice::Iter`, we implement our own iterator that doesn't
		// calculate the end pointer. This doesn't make creating page tables at
		// that address sound, but it avoids some easy to trigger
		// miscompilations.
		let ptr = self.entries.as_mut_ptr();
		(0..512).map(move |i| unsafe { &mut *ptr.add(i) })
	}

	/// Checks if the page table is empty (all entries are zero).
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.iter().all(|entry| entry.is_unused())
	}

	/// walk the page table, create missing tables, return mapped physical frame
	pub fn map_page(&self, _va: VAddr) {
		todo!()
	}
}

impl VAddr {
	#[inline]
	pub const fn new(addr: u64) -> VAddr {
		return Self::try_new(addr).expect("VA must be sign extended in 16 MSBs");
	}

	#[inline]
	pub const fn try_new(addr: u64) -> Option<VAddr> {
		let v = Self::new_truncate(addr);
		if v.0 == addr {
			Some(v)
		} else {
			None
		}
	}

	#[inline]
	pub const fn new_truncate(addr: u64) -> VAddr {
		// sign extend the upper bits
		VAddr(((addr << 16) as i64 >> 16) as u64)
	}

	#[inline]
	pub const fn as_u64(self) -> u64 {
		self.0
	}

	/// Converts the address to a raw pointer.
	#[cfg(target_pointer_width = "64")]
	#[inline]
	pub const fn as_ptr<T>(self) -> *const T {
		self.as_u64() as *const T
	}

	/// Converts the address to a mutable raw pointer.
	#[cfg(target_pointer_width = "64")]
	#[inline]
	pub const fn as_mut_ptr<T>(self) -> *mut T {
		self.as_ptr::<T>() as *mut T
	}

	/// Returns the 9-bit level 1 page table index.
	#[inline]
	pub const fn p1_index(self) -> usize {
		(self.0 >> 12) as usize
	}

	/// Returns the 9-bit level 2 page table index.
	#[inline]
	pub const fn p2_index(self) -> usize {
		(self.0 >> 12 >> 9) as usize
	}

	/// Returns the 9-bit level 3 page table index.
	#[inline]
	pub const fn p3_index(self) -> usize {
		(self.0 >> 12 >> 9 >> 9) as usize
	}

	/// Returns the 9-bit level 4 page table index.
	#[inline]
	pub const fn p4_index(self) -> usize {
		(self.0 >> 12 >> 9 >> 9 >> 9) as usize
	}
}

impl PAddr {
	#[inline]
	pub const fn new(addr: u64) -> Self {
		Self::try_new(addr).expect("PA shall not have more than 52 bits")
	}

	/// Creates a new physical address, throwing bits 52..64 away.
	#[inline]
	pub const fn new_truncate(addr: u64) -> PAddr {
		PAddr(addr % (1 << 52))
	}

	/// Tries to create a new physical address.
	/// Fails if any bits in the range 52 to 64 are set.
	#[inline]
	pub const fn try_new(addr: u64) -> Option<Self> {
		let p = Self::new_truncate(addr);
		if p.0 == addr {
			Some(p)
		} else {
			None
		}
	}

	#[inline]
	pub const fn as_u64(&self) -> u64 {
		self.0
	}
}

impl PTE {
	#[inline]
	pub const fn new() -> Self {
		PTE { entry: 0 }
	}

	#[inline]
	pub const fn is_unused(&self) -> bool {
		self.entry == 0
	}

	#[inline]
	pub fn set_unused(&mut self) -> bool {
		self.entry == 0
	}

	#[inline]
	pub const fn flags(&self) -> PTEFlags {
		// from_bits_truncate ignores undefined bits.
		PTEFlags::from_bits_truncate(self.entry)
	}

	#[inline]
	pub const fn addr(&self) -> PAddr {
		PAddr::new(self.entry & 0x000f_ffff_ffff_f000)
	}

	#[inline]
	pub fn clear(&mut self) {
		self.entry = 0;
	}
}

/// for x86_64, return the CR3 register. this is the **physical** address of the
/// page table root.
/// TODO: use page root in task struct instead of raw cr3
#[inline]
pub fn get_cr3() -> u64 {
	let cr3: u64;
	unsafe {
		asm!("mov {}, cr3", out(reg) cr3);
	}
	return cr3;
}

#[inline]
pub fn get_root() -> u64 {
	return P2V(get_cr3()).unwrap();
}
