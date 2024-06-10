use crate::arch::x86_64::arch_regs;
use crate::defs::*;
use crate::io::*;
use core::arch::asm;
use core::ptr;

/// currently only kernelSp and Context are important.
/// the task struct will be placed on the starting addr (low addr) of the kernel stack.
/// therefore we can retrive the task struct at anytime by masking the kernel stack
/// NOTE: we don't use repr(C) or repr(packed) here
pub struct Task {
	pub magic: u64,
	pub task_id: u32,
	/// note that this points to the stack bottom (low addr)
	pub kernel_stack: u64,
	// pub user_stack: u64,
	pub state: TaskState,
	pub context: arch_regs::Context64,
}

/// not to confuse with a integer TID. A TaskID identifies a task and __locate__
/// it. In this case the TaskID wraps around the task struct's address. The
/// reason why the scheduler doesn't directly store Box<Task> (or alike) is that
/// the smart pointer types automatically drops the owned values when their
/// lifetime end. For now want to have manual control of when, where and how I
/// drop the Task because there could be more plans than just freeing the memory
pub struct TaskId(u64);

impl TaskId {
	pub fn new(addr: u64) -> Self {
		Self { 0: addr }
	}

	pub unsafe fn get_task_ref(&self) -> &Task {
		&*(self.0 as *mut Task)
	}

	pub unsafe fn get_task_ref_mut(&self) -> &mut Task {
		&mut *(self.0 as *mut Task)
	}
}

#[derive(Debug)]
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
	let t = Task::current().unwrap();
	println!(
		"I am PID {}, kernel stack {:#X}, task state {:#X?}",
		t.task_id, t.kernel_stack, t.state
	);
	unsafe { asm!("cli; hlt") };
	panic!("should not reach");
}

extern "C" {
	pub fn context_swap(from_ctx: u64, to_ctx: u64);
	pub fn context_swap_to(to_ctx: u64);
}

impl Task {
	/// TODO implement a proper ::new. For now use settle_on_stack instead
	pub fn new(_id: u32, _kstack: u64, _func_ptr: u64) -> Self {
		panic!(
			"never ever try to manually create a task struct\n
			gather the parts and call Task::settle_on_stack() instead"
		)
	}

	/// unsafe because you have to make sure the stack pointer is valid
	/// i.e. allocated through KStackAllocator.
	#[inline(always)]
	pub unsafe fn settle_on_stack<'a>(stack_addr: u64, t: Task) -> &'a mut Task {
		ptr::write_volatile(stack_addr as *mut Task, t);
		return &mut *(stack_addr as *mut Task);
	}

	/// settle_on_stack and prepare_context must be called before switching to
	/// the task. TODO: combine them into one single API
	#[inline(always)]
	pub fn prepare_context(&mut self, entry: u64) {
		// this is like OOStuBS "toc_settle"
		let mut sp = self.get_init_kernel_sp();
		// FIXME this is ugly
		unsafe {
			sp -= 8;
			*(sp as *mut u64) = 0;
			sp -= 8;
			*(sp as *mut u64) = entry;
		}
		self.context.rsp = sp;
	}

	// stack pointer may have alignment requirement. We do 8 bytes
	#[inline(always)]
	fn get_init_kernel_sp(&self) -> u64 {
		let mut sp = self.kernel_stack + Mem::KERNEL_STACK_SIZE - 1;
		sp = sp & (!0b111);
		sp
	}

	pub fn current<'a>() -> Option<&'a mut Task> {
		let addr = arch_regs::get_sp() & !Mem::KERNEL_STACK_MASK;
		let t = unsafe { &mut *(addr as *mut Task) };
		if t.magic != Mem::KERNEL_STACK_TASK_MAGIC {
			return None;
		}
		return Some(t);
	}
}
