use crate::arch::x86_64::arch_regs;

/// currently only kernelSp and Context are important.
/// the task struct will be placed on the starting addr (low addr) of the kernel stack.
/// therefore we can retrive the task struct at anytime by masking the kernel stack
#[repr(C)]
#[repr(packed)]
pub struct Task {
	pub task_id: u32,
	pub kernel_stack: u64,
	pub user_stack: u64,
	pub context: arch_regs::Context64,
}
