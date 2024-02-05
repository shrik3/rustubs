pub struct Mem;
impl Mem {
	// units
	pub const K: usize = 1024;
	pub const M: usize = 1024 * Mem::K;
	pub const G: usize = 1024 * Mem::M;
	// physical memory
	pub const PHY_TOP: usize = 128 * Mem::M; // qemu defaults to 128 MiB phy Memory
	pub const PAGE_SIZE: usize = 0x1000;
	pub const PAGE_SHIFT: usize = 12;
	pub const PHY_PAGES: usize = Mem::PHY_TOP >> Mem::PAGE_SHIFT;
	pub const PAGE_MASK: u64 = 0xfff;
}
