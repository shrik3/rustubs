use core::arch::asm;

/// arch specific registers
#[repr(C)]
#[repr(packed)]
#[derive(Debug)]
pub struct Context64 {
	pub rbx: u64,
	pub r12: u64,
	pub r13: u64,
	pub r14: u64,
	pub r15: u64,
	pub rbp: u64,
	pub rsp: u64,
	pub fpu: [u8; 108],
}

/// arch specific registers
#[repr(C)]
#[repr(packed)]
#[derive(Debug)]
pub struct TrapFrame {
	pub r11: u64,
	pub r10: u64,
	pub r9: u64,
	pub r8: u64,
	pub rsi: u64,
	pub rdi: u64,
	pub rdx: u64,
	pub rcx: u64,
	pub rax: u64,
	pub err_code: u64,
}

// this will get the current (kernel) stack pointer
#[inline]
pub fn get_sp() -> u64 {
	let sp: u64;
	unsafe {
		asm!("mov {}, rsp", out(reg) sp);
	}
	return sp;
}
