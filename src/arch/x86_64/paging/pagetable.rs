//! basic paging support.
//! code derived from the
//! [x86_64 crate](https://docs.rs/x86_64/latest/src/x86_64/addr.rs.html)
//! see ATTRIBUTIONS

use bitflags::bitflags;

#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct Pagetable {
	pub entries: [PTE; Self::ENTRY_COUNT],
}

#[derive(Clone)]
#[repr(transparent)]
pub struct PTE {
	pub entry: u64,
}

bitflags! {
#[derive(Debug, Copy, Clone)]
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
	pub fn set_unused(&mut self) {
		self.entry = 0;
	}

	#[inline]
	pub const fn flags(&self) -> PTEFlags {
		// from_bits_truncate ignores undefined bits.
		PTEFlags::from_bits_truncate(self.entry)
	}

	#[inline]
	pub const fn addr(&self) -> u64 {
		self.entry & 0x000f_ffff_ffff_f000
	}

	#[inline]
	pub fn set(&mut self, pa: u64, flags: PTEFlags) {
		self.entry = pa | flags.bits();
	}
}

const ID_MASK: u64 = 0x1ff;
#[inline]
pub fn p4idx(addr: u64) -> u16 {
	((addr >> 12 >> 9 >> 9 >> 9) & ID_MASK) as u16
}
#[inline]
pub fn p3idx(addr: u64) -> u16 {
	((addr >> 12 >> 9 >> 9) & ID_MASK) as u16
}
#[inline]
pub fn p2idx(addr: u64) -> u16 {
	((addr >> 12 >> 9) & ID_MASK) as u16
}
#[inline]
pub fn p1idx(addr: u64) -> u16 {
	((addr >> 12) & ID_MASK) as u16
}
