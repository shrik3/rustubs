use crate::arch::x86_64::is_int_enabled;
use crate::io::*;
use crate::machine::interrupt::{irq_restore, irq_save};
use crate::proc::sync::*;
use crate::proc::task::*;
use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::cell::SyncUnsafeCell;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use lazy_static::lazy_static;
use spin::Mutex;

pub static GLOBAL_SCHEDULER: L3SyncCell<Scheduler> = L3SyncCell::new(Scheduler::new());
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

// TODO the lifetime here is pretty much broken. Fix this later
// the scheduler should be a per-cpu instance and it shall not lock.
// Because the `do_schedule` does not return to release the lock
pub struct Scheduler {
	pub run_queue: VecDeque<TaskId>,
	pub need_schedule: bool,
}

impl Scheduler {
	pub const MIN_TASK_CAP: usize = 16;
	pub const fn new() -> Self {
		// btw. try_with_capacity is an unstable feature.
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
		// try to remove all occurence of tid in the run_queue maybe do
		// something special if the task is in the wait queue but we are not
		// there yet.
		todo!("not implemented");
	}

	/// unsafe because this must be called on a linearization point on Epilogue
	/// level (l2); It will check the NEED_RESCHEDULE flag.
	pub unsafe fn try_reschedule() {
		// this assert doesn't check if you own the L2, but at least a sanity
		// check.
		assert!(is_int_enabled());
		assert!(IS_L2_AVAILABLE());
		// TODO maybe refine memory ordering here
		let r = NEED_RESCHEDULE.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
		if r != Ok(true) {
			return;
		}
		Self::do_schedule();
	}

	pub unsafe fn do_schedule_coperative() {}

	// pop front, push back
	pub unsafe fn do_schedule() {
		let me = Task::current().unwrap();
		let next_task;
		let next_tid;
		L3_CRITICAL! {
			// the L3 critical section
			// make sure we drop the mutable borrow before doing context swap
			let sched = GLOBAL_SCHEDULER.l3_get_ref_mut();
			next_tid = sched.run_queue.pop_front().expect("empty run queue, how?");
			next_task = next_tid.get_task_ref_mut();
			sched.run_queue.push_back(next_tid);
		}
		if me.pid == next_task.pid {
			return;
		}
		use alloc::format;
		unsafe {
			context_swap(
				&(me.context) as *const _ as u64,
				&(next_task.context) as *const _ as u64,
			);
		}
	}

	// like do_schedule but we there is no running context to save
	pub unsafe fn kickoff() {
		let tid;
		let first_task;
		L3_CRITICAL! {
			let sched = GLOBAL_SCHEDULER.l3_get_ref_mut();
			tid = sched
				.run_queue
				.pop_front()
				.expect("run queue empty, can't start");
			first_task = tid.get_task_ref_mut();
			sched.run_queue.push_back(tid);
		}
		unsafe {
			context_swap_to(&(first_task.context) as *const _ as u64);
		}
	}
}
