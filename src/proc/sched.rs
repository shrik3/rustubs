use crate::io::*;
use crate::proc::task::*;
use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::Mutex;
// the global scheduler takes a spinlock (will change later). Must be extra
// careful with it: never do context swap before releasing the lock on scheduler,
// otherwise the next task won't be able to aquire the lock again.
lazy_static! {
	pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}
// TODO the lifetime here is pretty much broken. Fix this later
// the scheduler should be a per-cpu instance and it shall not lock.
// Because the `do_schedule` does not return to release the lock
pub struct Scheduler {
	pub run_queue: VecDeque<TaskId>,
}

impl Scheduler {
	pub const MIN_TASK_CAP: usize = 16;
	pub fn new() -> Self {
		Self {
			// btw. try_with_capacity is an unstable feature.
			run_queue: VecDeque::try_with_capacity(16).unwrap(),
		}
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

	// pop front, push back
	pub fn do_schedule() {
		// TODO: remove this spinlock, because we should protect the scheduler
		// with irq_save/restore
		if SCHEDULER.is_locked() {
			panic!("scheduler lock has been taken, something wrong");
		}
		let mut lock_guard = SCHEDULER.lock();
		let next_tid = lock_guard
			.run_queue
			.pop_front()
			.expect("empty run queue, how?");
		let next_task = next_tid.get_task_ref_mut();
		let me = Task::current().unwrap();
		lock_guard.run_queue.push_back(next_tid);
		if me.pid == next_task.pid {
			return;
		}
		// make sure we release the scheduler lock before doing context
		// swap
		drop(lock_guard);
		unsafe {
			context_swap(
				&(me.context) as *const _ as u64,
				&(next_task.context) as *const _ as u64,
			);
		}
	}

	// like do_schedule but we there is no running context to save
	pub fn kickoff() {
		if SCHEDULER.is_locked() {
			panic!("scheduler lock has been taken, something wrong");
		}
		let mut lock_guard = SCHEDULER.lock();
		let t = lock_guard
			.run_queue
			.pop_front()
			.expect("run queue empty, can't start");
		let first_task = t.get_task_ref_mut();
		lock_guard.run_queue.push_back(t);
		drop(lock_guard);
		unsafe {
			context_swap_to(&(first_task.context) as *const _ as u64);
		}
	}
}
