use crate::arch::x86_64::is_int_enabled;
use crate::machine::interrupt::{irq_restore, irq_save};
use crate::proc::sync::*;
use crate::proc::task::*;
use alloc::collections::VecDeque;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
pub static GLOBAL_SCHEDULER: L2Sync<Scheduler> = L2Sync::new(Scheduler::new());
/// A global flag indicating whether reschedule is required.
pub static NEED_RESCHEDULE: AtomicBool = AtomicBool::new(false);

/// set NEED_RESCHEDULE to true regardless its value; return the previous state.
#[inline(always)]
#[allow(non_snake_case)]
pub fn SET_NEED_RESCHEDULE() -> bool {
	NEED_RESCHEDULE.swap(true, Ordering::Relaxed)
}

/// set NEED_RESCHEDULE to false regardless its value; return the previous
/// state.
#[inline(always)]
#[allow(non_snake_case)]
pub fn CLEAR_NEED_RESCHEDULE() -> bool {
	NEED_RESCHEDULE.swap(false, Ordering::Relaxed)
}

pub struct Scheduler {
	pub run_queue: VecDeque<TaskId>,
	pub need_schedule: bool,
}

impl Scheduler {
	pub const MIN_TASK_CAP: usize = 16;
	pub const fn new() -> Self {
		return Self {
			run_queue: VecDeque::new(),
			need_schedule: false,
		};
	}

	// maybe we reject inserting existing tasks?
	pub fn insert_task(&mut self, tid: TaskId) {
		self.run_queue.push_back(tid);
	}

	pub fn try_remove(&mut self, _tid: TaskId) {
		todo!("not implemented");
	}

	/// unsafe because this must be called on a linearization point on Epilogue
	/// level (l2); It will check the NEED_RESCHEDULE flag.
	pub unsafe fn try_reschedule() {
		// this assert doesn't check if you own the L2, but at least a sanity
		// check.
		assert!(is_int_enabled());
		// TODO maybe refine memory ordering here
		let r = NEED_RESCHEDULE.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
		if r != Ok(true) {
			return;
		}
		Self::do_schedule();
	}

	/// do_schedule is only called from epilogue level, so we don't need to lock
	/// here. For cooperative scheduling call [yield_cpu] instead.
	pub unsafe fn do_schedule() {
		let me = Task::current().unwrap();
		let next_task;
		let next_tid;
		{
			let r = irq_save();
			// begin L3 critical section
			// make sure we drop the mutable borrow before doing context swap
			let sched = GLOBAL_SCHEDULER.get_ref_mut_unguarded();
			if sched.run_queue.is_empty() && me.state == TaskState::Run {
				// I'm the only one, just return;
				irq_restore(r);
				return;
			}
			next_tid = sched.run_queue.pop_front().expect("no runnable task");
			next_task = next_tid.get_task_ref_mut();
			assert_eq!(next_task.state, TaskState::Run);
			if me.state == TaskState::Run {
				sched.run_queue.push_back(me.taskid());
			}
			// end L3 critical section
			irq_restore(r);
		}
		if me.taskid() == next_task.taskid() {
			return;
		}
		unsafe {
			context_swap(
				&(me.context) as *const _ as u64,
				&(next_task.context) as *const _ as u64,
			);
		}
	}

	/// guards do_schedule and makes sure it's also sequentialized at L2. Must
	/// not call this in interrupt context
	pub fn yield_cpu() {
		assert!(is_int_enabled());
		ENTER_L2();
		unsafe {
			Self::do_schedule();
		}
		LEAVE_L2();
	}

	// like do_schedule but we there is no running context to save
	pub unsafe fn kickoff() {
		let tid;
		let first_task;
		let irq = irq_save();
		// must not lock the GLOBAL_SCHEDULER here because we never return.
		// well, the "LEAVE_L2" call in the task entries logically release
		// the GLOBAL_SCHEDULER but semantically that's too weird
		let sched = GLOBAL_SCHEDULER.get_ref_mut_unguarded();
		tid = sched
			.run_queue
			.pop_front()
			.expect("run queue empty, can't start");
		first_task = tid.get_task_ref_mut();
		irq_restore(irq);
		// kickoff simulates a do_schedule, so we need to enter l2 here.
		// new tasks must leave l2 explicitly on their first run
		ENTER_L2();
		unsafe {
			context_swap_to(&(first_task.context) as *const _ as u64);
		}
	}
}
