extern "C" {
	fn ___KERNEL_PM_START__();
	fn ___KERNEL_PM_END__();
	fn ___BSS_START__();
	fn ___BSS_END__();
}

// ANY ADDRESS FROM PHYSICAL MAPPING IS UNSAFE BECAUSE THE LOW MEMORY MAPPING
// WILL BE DROPPED FOR USERSPACE
// TODO: create VMAs in the MM struct
#[inline]
pub unsafe fn pmap_kernel_start() -> u64 {
	___KERNEL_PM_START__ as u64
}

#[inline]
pub unsafe fn pmap_kernel_end() -> u64 {
	___KERNEL_PM_END__ as u64
}

#[inline]
pub unsafe fn vmap_kernel_start() -> u64 {
	pmap_kernel_start() + Mem::KERNEL_OFFSET
}

#[inline]
pub unsafe fn vmap_kernel_end() -> u64 {
	pmap_kernel_end() + Mem::KERNEL_OFFSET
}
// ABOVE ONLY VALID BEFORE DROPPING LOWER MEMORY MAPPING -----//

#[inline]
pub fn bss_start() -> u64 {
	return ___BSS_START__ as u64;
}

#[inline]
pub fn bss_end() -> u64 {
	return ___BSS_END__ as u64;
}

#[inline]
pub fn roundup_4k(addr: u64) -> u64 {
	return (addr + 0xfff) & !0xfff;
}

#[inline]
pub fn rounddown_4k(addr: u64) -> u64 {
	return addr & !0xfff;
}

pub struct Mem;
impl Mem {
	// units
	pub const K: u64 = 1024;
	pub const M: u64 = 1024 * Mem::K;
	pub const G: u64 = 1024 * Mem::M;
	// 4 lv 4K paging
	pub const PAGE_SIZE: u64 = 0x1000;
	pub const PAGE_SHIFT: u64 = 12;
	pub const PAGE_MASK: u64 = 0xfff;
	pub const L0_SHIFT: u8 = 39;
	pub const L0_MASK: u64 = 0x1ff << Mem::L0_SHIFT;
	pub const L1_SHIFT: u8 = 30;
	pub const L1_MASK: u64 = 0x1ff << Mem::L1_SHIFT;
	pub const L2_SHIFT: u8 = 21;
	pub const L2_MASK: u64 = 0x1ff << Mem::L2_SHIFT;
	pub const L3_SHIFT: u8 = 12;
	pub const L3_MASK: u64 = 0x1ff << Mem::L3_SHIFT;
	// 64 GiB available memory
	pub const MAX_PHY_MEM: u64 = 0x1000000000;
	// we should have at least 64 MiB free physical memory (excluding the kernel it self)
	pub const MIN_PHY_MEM: u64 = 64 * Self::M;
	// size of frame allocator bitmap: number of physical frames / 8 for 128M
	// memory (37268) 4k pages, 37268 bits are needed, hence
	// 4096 bytes, exactly one page!
	pub const ID_MAP_START: u64 = 0xffff_8000_0000_0000;
	pub const ID_MAP_END: u64 = 0xffff_8010_0000_0000;
	// kernel image:0xffff_8020_0000_0000 ~ 0xffff_802f_0000_0000;
	pub const KERNEL_OFFSET: u64 = 0xffff_8020_0000_0000;
	// kernel heap: 0xffff_8030_0000_0000 ~ 0xffff_803f_0000_0000;
	// (64 GiB)
	pub const KERNEL_HEAP_START: u64 = 0xffff_8030_0000_0000;
	pub const KERNEL_HEAP_END: u64 = 0xffff_8040_0000_0000;
	// unlike the initial "thread" that has 64K stack, new tasks have 4 pages of
	// kernel stack.
	pub const KERNEL_STACK_SIZE: u64 = 0x4000;
	pub const KERNEL_STACK_MASK: u64 = Self::KERNEL_STACK_SIZE - 1;
	pub const KERNEL_STACK_TASK_MAGIC: u64 = 0x1A2B3C4D5E6F6969;
}

// convert VA <-> PA wrt. the kernel id mapping
// from 0xffff_8000_0000_0000 ~ 0xffff_800f_ffff_ffff virtual
// to 0x0 ~ 0xf_ffff_ffff physical (64G)
#[allow(non_snake_case)]
#[inline]
pub fn V2P(va: u64) -> Option<u64> {
	if va >= Mem::ID_MAP_END || va < Mem::ID_MAP_START {
		return None;
	}
	return Some(va - Mem::ID_MAP_START);
}

#[allow(non_snake_case)]
#[inline]
pub fn P2V(pa: u64) -> Option<u64> {
	if pa >= Mem::MAX_PHY_MEM {
		return None;
	}
	return Some(pa + Mem::ID_MAP_START);
}
