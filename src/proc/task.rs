use crate::arch::x86_64::arch_regs;
use crate::defs::*;
use crate::io::*;
use core::arch::asm;

/// currently only kernelSp and Context are important.
/// the task struct will be placed on the starting addr (low addr) of the kernel stack.
/// therefore we can retrive the task struct at anytime by masking the kernel stack
#[repr(C)]
#[repr(packed)]
pub struct Task {
	pub magic: u64,
	pub task_id: u32,
	pub kernel_stack: u64,
	// pub user_stack: u64,
	pub context: arch_regs::Context64,
	pub state: TaskState,
}

pub enum TaskState {
	Run,
	Block,
	Dead,
	Eating,
	Purr,
	Meow,
	Angry,
}

#[no_mangle]
pub extern "C" fn _task_entry() -> ! {
	println!("I'm Mr.Meeseeks, look at me~");
	unsafe { asm!("hlt") };
	panic!("shoud not reach");
}

extern "C" {
	fn context_swap(from_ctx: u64, to_ctx: u64);
	fn context_swap_to(to_ctx: u64);
}

impl Task {
	/// create new task. This is tricky because the task struct sits at the
	/// bottom of the kernel stack, so that you can get the current task by
	/// masking the stack pointer.
	/// 1. allocate the kernel stack with predefined size and make sure the
	///    address is properly aligned
	/// 2. cast a task struct onto the stack
	/// 3. set whatever fields necessary in the task struct, including a magic
	/// 4. create a new stack frame, and update the stack pointer in the context,
	///	   so that when first swapped to, the task will immediately "return to"
	///	   the _func_ptr
	pub fn new(_id: u32, _func_ptr: u64) -> Self {
		todo!()
	}
}
