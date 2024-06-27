//! system level definitions

/// multiboot magic value, it must be 0x2BAD8002. This value is set at runtime
/// by init asm code. CARE: the release build will not treat these as volatile
/// and they may hardcode the initial value (0) because they sees "static const
/// zero". Therefore when reading them must do a volatile read.
#[no_mangle]
pub static mb_magic: u64 = 0;
/// _physical_ address of multiboot info block. This value is set at run time by
/// init asm code. Both values are safe to read, but [mb_info_pm_addr] must be
/// converted to the virtual mapping via P2V before dereferencing.
#[no_mangle]
pub static mb_info_pm_addr: u64 = 0;

#[inline]
pub fn roundup_4k(addr: u64) -> u64 {
	(addr + 0xfff) & !0xfff
}

#[inline]
pub fn rounddown_4k(addr: u64) -> u64 {
	addr & !0xfff
}

#[inline]
pub fn is_aligned_4k(addr: u64) -> bool {
	(addr & 0xfff) == 0
}

/// memory definitions
pub mod Mem {
	// units
	pub const K: u64 = 1024;
	pub const M: u64 = 1024 * K;
	pub const G: u64 = 1024 * M;
	// 4 lv 4K paging
	pub const PAGE_SIZE: u64 = 0x1000;
	pub const PAGE_SHIFT: u64 = 12;
	pub const PAGE_MASK: u64 = 0xfff;
	// 64 GiB available memory
	pub const MAX_PHY_MEM: u64 = 0x1000000000;
	// we should have at least 64 MiB free physical memory (excluding the kernel it self)
	pub const MIN_PHY_MEM: u64 = 64 * M;
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
	pub const KERNEL_STACK_MASK: u64 = KERNEL_STACK_SIZE - 1;
	pub const KERNEL_STACK_TASK_MAGIC: u64 = 0x1A2B3C4D5E6F6969;
	// user (psuedo)
	pub const USER_STACK_START: u64 = 0x0000_7000_0000_0000;
	pub const USER_STACK_SIZE: u64 = 8 * M;
}

// TODO use a consistent naming convention for extern symbols
pub mod ExternSyms {
	extern "C" {
		pub fn ___KERNEL_PM_START__();
		pub fn ___KERNEL_PM_END__();
		pub fn ___BSS_START__();
		pub fn ___BSS_END__();
		pub fn ___RAMFS_START__();
		pub fn ___RAMFS_END__();
		/// a chunk (8M) of reserved memory, optionally used by the stack based
		/// physical frame allocator. This naive pma is deprecated, and you must not
		/// use this symbol unless you adjust the startup code to reserve
		/// memory of cooresponding size and alignment. This is deprecated
		pub fn ___FREE_PAGE_STACK__();
	}
	#[cfg(target_arch = "x86_64")]
	pub use crate::arch::x86_64::ExternSyms::*;
}

#[cfg(target_arch = "x86_64")]
pub mod HWDefs {
	/// number of entries in IDT
	pub const IDT_CAPACITY: usize = 256;
	/// 32 exceptions + 16 irqs from PIC = 48 valid interrupts
	pub const IDT_VALID: usize = 48;
	/// size of interrupt handler wrapper routine (vector)
	pub const VECTOR_SIZE: usize = 16;
}

pub mod Limits {
	// initialize some queue structs with a reserved capacity to avoid runtime
	// allocation
	pub const SEM_WAIT_QUEUE_MIN_CAP: usize = 16;
	pub const SCHED_RUN_QUEUE_MIN_CAP: usize = 24;
}

/// convert VA <-> PA wrt. the kernel id mapping
/// from 0xffff_8000_0000_0000 ~ 0xffff_800f_ffff_ffff virtual
/// to 0x0 ~ 0xf_ffff_ffff physical (64G)
#[allow(non_snake_case)]
#[inline]
pub const fn V2P(va: u64) -> Option<u64> {
	if va >= Mem::ID_MAP_END || va < Mem::ID_MAP_START {
		return None;
	}
	return Some(va - Mem::ID_MAP_START);
}

/// physical address to virtual. reverse of [V2P]
#[allow(non_snake_case)]
#[inline]
pub const fn P2V(pa: u64) -> Option<u64> {
	if pa >= Mem::MAX_PHY_MEM {
		return None;
	}
	return Some(pa + Mem::ID_MAP_START);
}

/// interrut numbers. Not complete, add more when needed
/// (see docs/interrupt.txt)
pub struct IntNumber {}
impl IntNumber {
	pub const PAGEFAULT: u16 = 0xe;
	pub const TIMER: u16 = 0x20;
	pub const KEYBOARD: u16 = 0x21;
	pub const SYSCALL: u16 = 0x80;
}
