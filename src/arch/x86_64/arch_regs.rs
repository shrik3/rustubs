//! both [Context64] and [TrapFrame] define architecture specific registers and
//! combine into the full execution context of a thread.
//! [Context64] includes the callee saved registers plus FP state, and
//! [TrapFrame] includes caller saved registers.

use core::arch::asm;
#[repr(C)]
#[repr(packed)]
#[derive(Debug)]
/// the Context64 is part of the task struct; it's saved and restored explicitly
/// on context swap.
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

impl Default for Context64 {
	fn default() -> Context64 {
		Context64 {
			rbx: 0,
			r12: 0,
			r13: 0,
			r14: 0,
			r15: 0,
			rbp: 0,
			rsp: 0,
			fpu: [0; 108],
		}
	}
}

/// `TrapFrame` is saved and restored by the interrupt handler assembly code
/// upon interrupt entry and exit.
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
	/// for some exceptions, the CPU automatically pushes an error code (see
	/// `docs/interrupt.txt`) to the stack. For those who don't have error code,
	/// we manually push a dummy value (0)
	pub err_code: u64,
}

/// get the current stack pointer
#[inline]
pub fn get_sp() -> u64 {
	let sp: u64;
	unsafe {
		asm!("mov {}, rsp", out(reg) sp);
	}
	return sp;
}
