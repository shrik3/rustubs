pub mod arch_regs;
pub mod interrupt;
pub mod io_port;
pub mod mem;
pub mod misc;
pub mod paging;
use core::arch::asm;

pub const RFLAGS_IF_MASK: u64 = 1 << 9;
#[inline]
pub fn read_rflags() -> u64 {
	let rflags;
	unsafe {
		asm!("pushfq; pop {}", out(reg) rflags);
	}
	rflags
}

pub fn is_int_enabled() -> bool {
	let rf = read_rflags();
	return (rf & RFLAGS_IF_MASK) != 0;
}
