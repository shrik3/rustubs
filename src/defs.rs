pub struct Mem;
pub struct VAddr(u64);

impl Mem {
	// units
	pub const K: usize = 1024;
	pub const M: usize = 1024 * Mem::K;
	pub const G: usize = 1024 * Mem::M;
	// physical memory layout: qemu defaults to 128 MiB phy Memory
	pub const PHY_TOP: usize = 128 * Mem::M;
	// 4 lv 4K paging
	pub const PAGE_SIZE: usize = 0x1000;
	pub const PAGE_SHIFT: usize = 12;
	pub const PAGE_MASK: u64 = 0xfff;
	pub const L0_SHIFT: u8 = 39;
	pub const L0_MASK: u64 = 0x1ff << Mem::L0_SHIFT;
	pub const L1_SHIFT: u8 = 30;
	pub const L1_MASK: u64 = 0x1ff << Mem::L1_SHIFT;
	pub const L2_SHIFT: u8 = 21;
	pub const L2_MASK: u64 = 0x1ff << Mem::L2_SHIFT;
	pub const L3_SHIFT: u8 = 12;
	pub const L3_MASK: u64 = 0x1ff << Mem::L3_SHIFT;
	pub const PHY_PAGES: usize = Mem::PHY_TOP >> Mem::PAGE_SHIFT;
	// size of frame allocator bitmap: number of physical frames / 8 for 128M
	// memory (37268) 4k pages, 37268 bits are needed, hence
	// 4096 bytes, exactly one page!
	pub const PHY_BM_SIZE: usize = Mem::PHY_PAGES >> 3;
}

impl VAddr {
	pub fn roundup_4k(&self) {
		todo!()
	}
	pub fn rounddown_4k(&self) {
		todo!()
	}
	pub fn page_number(&self) -> u64 {
		self.0 >> Mem::PAGE_SHIFT
	}
}

// PHY_TOP			128M
// ~				free frames
// PMA::bitmap		+ PHY_BM_SIZE
// ~				___KERNEL_END__
// KERNEL IMAGE
// KERNEL START		1 M
