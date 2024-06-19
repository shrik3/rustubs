use crate::arch::x86_64::arch_regs::TrapFrame;
use crate::io::*;
use core::arch::asm;

/// handle page fault: for now we only check if the faulting addr is within
/// kernel heap range.
/// TODO: improve this later
pub fn page_fault_handler(frame: &mut TrapFrame, fault_addr: u64) {
	let err_code = frame.err_code;
	println!("pagefault @ {:#X}, err {:#X?}", fault_addr, err_code);
	unsafe { asm!("hlt") };
}

/// for x86_64, return the CR3 register.
#[inline]
pub fn get_fault_addr() -> u64 {
	let cr2: u64;
	unsafe {
		asm!("mov {}, cr2", out(reg) cr2);
	}
	return cr2;
}
