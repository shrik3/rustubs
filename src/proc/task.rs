use crate::arch::x86_64::arch_regs::Context64;
use crate::arch::x86_64::{arch_regs, is_int_enabled};
use crate::mm::vmm::{VMArea, VMMan, VMPerms, VMType};
use crate::mm::KSTACK_ALLOCATOR;
use crate::proc::sched::GLOBAL_SCHEDULER;
use crate::proc::sync::bellringer::{BellRinger, Sleeper};
use crate::{defs::*, Scheduler};
use alloc::collections::VecDeque;
use alloc::string::String;
use core::ops::Range;
use core::ptr;
use core::str::FromStr;
/// currently only kernelSp and Context are important.
/// the task struct will be placed on the starting addr (low addr) of the kernel stack.
/// therefore we can retrive the task struct at anytime by masking the kernel stack
/// NOTE: we assume all fields in [Task] are only modified by the task itself,
/// i.e. no task should modify another task's state. (this may change though, in
/// which case we will need some atomics)
/// TODO: the mm is heap allocated object (vec of vmas). But the task struct
/// doesn't have a lifetime. Must cleanup the memory used by the mm itself when
/// exiting a task.
#[repr(C)]
pub struct Task {
	pub magic: u64,
	pub pid: u32,
	/// note that this points to the stack bottom (low addr)
	pub kernel_stack: u64,
	pub mm: VMMan,
	// pub user_stack: u64,
	pub state: TaskState,
	pub context: arch_regs::Context64,
}

/// not to confuse with a integer TID. A TaskID identifies a task and __locate__
/// it. In this case the TaskID wraps around the task struct's address. The
/// reason why the scheduler doesn't directly store `Box<Task>` (or alike) is that
/// the smart pointer types automatically drops the owned values when their
/// lifetime end. For now want to have manual control of when, where and how I
/// drop the Task because there could be more plans than just freeing the memory
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
	pub fn new(addr: u64) -> Self { Self(addr) }

	pub fn get_task_ref(&self) -> &Task {
		return unsafe { &*(self.0 as *mut Task) };
	}

	pub fn get_task_ref_mut(&self) -> &mut Task {
		return unsafe { &mut *(self.0 as *mut Task) };
	}
}

/// currently don't differentiate between running and ready states because the
/// scheduler push the next task to the back of the queue. i.e. the running task
/// is also "ready" in the run_queue
#[derive(Debug, PartialEq)]
pub enum TaskState {
	Run,
	Wait,
	Block,
	Dead,
	Eating,
	Purr,
	Meow,
	Angry,
}

extern "C" {
	pub fn context_swap(from_ctx: u64, to_ctx: u64);
	pub fn context_swap_to(to_ctx: u64);
}

// NOTE Task struct is manually placed on the stack, new() or default() is not
// provided.
impl Task {
	/// unsafe because you have to make sure the stack pointer is valid
	/// i.e. allocated through KStackAllocator.
	#[inline(always)]
	unsafe fn settle_on_stack<'a>(stack_addr: u64, t: Task) -> &'a mut Task {
		ptr::write_volatile(stack_addr as *mut Task, t);
		return &mut *(stack_addr as *mut Task);
	}

	/// settle_on_stack and prepare_context must be called before switching to
	/// the task. TODO: combine them into one single API
	#[inline(always)]
	fn prepare_context(&mut self, entry: u64) {
		let mut sp = self.get_init_kernel_sp();
		unsafe {
			sp -= 8;
			*(sp as *mut u64) = 0;
			sp -= 8;
			*(sp as *mut u64) = entry;
		}
		self.context.rsp = sp;
	}

	/// get kernel stack top (high addr) to initialize the new task Note that
	/// there are often alignment requirements of stack pointer. We do
	/// 8 bytes here
	#[inline(always)]
	fn get_init_kernel_sp(&self) -> u64 {
		let mut sp = self.kernel_stack + Mem::KERNEL_STACK_SIZE;
		sp &= !0b111;
		sp
	}

	/// return a reference of the current running task struct. Return none of
	/// the magic number is currupted on the kernel stack, this is because
	/// 1. the task struct is not currectly put on the stack
	/// 2. trying to get the current of the initial task, who has no task struct
	///       on the stack
	/// 3. the stack is corrupted (due to e.g. stack overflow)
	///
	/// TODO add a canary also at the end of the task struct and check it.
	pub fn current<'a>() -> Option<&'a mut Task> {
		let addr = arch_regs::get_sp() & !Mem::KERNEL_STACK_MASK;
		let t = unsafe { &mut *(addr as *mut Task) };
		if t.magic != Mem::KERNEL_STACK_TASK_MAGIC {
			return None;
		}
		return Some(t);
	}

	#[inline]
	pub fn taskid(&self) -> TaskId { TaskId::new(self as *const _ as u64) }

	/// a task may be present in multiple wait rooms; this is logically not
	/// possible at the moment, but would be necessary for stuffs like EPoll.
	/// require manual attention for sync
	pub unsafe fn curr_wait_in(wait_room: &mut VecDeque<TaskId>) {
		let t = Task::current().unwrap();
		debug_assert_ne!(t.state, TaskState::Wait);
		t.state = TaskState::Wait;
		wait_room.push_back(t.taskid());
	}

	/// does not lock the GLOBAL_SCHEDULER, the caller is responsible of doing
	/// that, e.g. call task.wakeup() from epilogue
	pub unsafe fn wakeup(&mut self) {
		if self.state != TaskState::Wait {
			// already awake. why? I don't know.
			return;
		}
		// TODO: makesure you don't put a task in the run queue more than once.
		self.state = TaskState::Run;
		let sched = GLOBAL_SCHEDULER.get_ref_mut_unguarded();
		sched.insert_task(self.taskid());
	}

	pub fn nanosleep(&mut self, ns: u64) {
		debug_assert!(self.state == TaskState::Run);
		self.state = TaskState::Wait;
		BellRinger::check_in(Sleeper::new(self.taskid(), ns));
		debug_assert!(is_int_enabled());
		Scheduler::yield_cpu();
	}

	/// create a kernel thread, you need to add it to the scheduler run queue
	/// manually
	pub fn create_task(pid: u32, entry: u64) -> TaskId {
		let sp = unsafe { KSTACK_ALLOCATOR.lock().allocate() };
		let tid = TaskId::new(sp);
		println!("new task on {:#X}", sp);
		let nt = unsafe {
			Task::settle_on_stack(
				sp,
				Task {
					magic: Mem::KERNEL_STACK_TASK_MAGIC,
					pid,
					kernel_stack: sp,
					state: TaskState::Run,
					context: Context64::default(),
					mm: VMMan::new(),
				},
			)
		};
		// KERNEL ID MAPPING
		nt.mm.vmas.push(VMArea {
			vm_range: Range::<u64> {
				start: Mem::ID_MAP_START,
				end: Mem::ID_MAP_END,
			},
			tag: String::from_str("KERNEL IDMAP").unwrap(),
			user_perms: VMPerms::NONE,
			backing: VMType::ANOM,
		});
		// KERNEL
		nt.mm.vmas.push(VMArea {
			vm_range: Range::<u64> {
				start: Mem::KERNEL_OFFSET,
				end: Mem::KERNEL_OFFSET + 64 * Mem::G,
			},
			tag: String::from_str("KERNEL").unwrap(),
			user_perms: VMPerms::NONE,
			backing: VMType::ANOM,
		});
		// KERNEL
		nt.mm.vmas.push(VMArea {
			vm_range: Range::<u64> {
				start: Mem::USER_STACK_START,
				end: Mem::USER_STACK_START + Mem::USER_STACK_SIZE,
			},
			tag: String::from_str("USER STACK").unwrap(),
			user_perms: VMPerms::R | VMPerms::W,
			backing: VMType::ANOM,
		});
		nt.prepare_context(entry);
		tid
	}
}
