use crate::arch::x86_64::is_int_enabled;
use crate::proc::task::{Task, TaskId};
use crate::Scheduler;
use alloc::collections::VecDeque;
use core::sync::atomic::Ordering;
use core::{cell::SyncUnsafeCell, sync::atomic::AtomicU64};

use super::L2_GUARD;
pub trait Semaphore<T, E>
where
	T: ResourceMan<E>,
	E: Copy + Clone,
{
	/// Probeer (try): the consumer end, tries to get resource, blocks on empty
	fn p(&self) -> Option<E>;
	/// Verhoog (increment): the producer end, increments the resource and must
	/// not block
	fn v(&self, e: E);
	/// must not block
	fn is_empty(&self) -> bool;
	/// must not block
	fn is_full(&self) -> bool;
	/// if the semaphore is also to be accessed in the epilogue level, the L2
	/// lock is already acquired.
	unsafe fn p_unguarded(&self) -> Option<E> {
		return None;
	}
	#[allow(unused_variables)]
	unsafe fn v_unguarded(&self, e: E) {}
}

/// wherever resoure management behind semaphore must provide get and insert
/// function. They do not need to be atomic. Normaly they only needs to be
/// wrappers for e.g. enque and deque.
///
/// TODO: implement a "reserve" optional "reserve" for ResourceMan<E>
pub trait ResourceMan<E>
where E: Copy + Clone
{
	fn get_resource(&mut self) -> Option<E>;
	fn insert_resource(&mut self, e: E) -> bool;
}

impl<E> ResourceMan<E> for VecDeque<E>
where E: Copy + Clone
{
	fn get_resource(&mut self) -> Option<E> {
		self.pop_front()
	}
	fn insert_resource(&mut self, e: E) -> bool {
		self.push_back(e);
		// well, it seems that the vectorDeque has no failing case for
		// push_back. TODO set a hard capacity limit
		true
	}
}

pub struct SleepSemaphore<T> {
	pub resource_pool: SyncUnsafeCell<T>,
	pub sema: AtomicU64,
	// the wait_room must be synchronized at level 3 (or???)
	// TODO make a type alias for VecDeque<TaskId>
	pub wait_room: SyncUnsafeCell<VecDeque<TaskId>>,
}

impl<T> SleepSemaphore<T> {
	pub const fn new(t: T) -> Self {
		Self {
			resource_pool: SyncUnsafeCell::new(t),
			sema: AtomicU64::new(0),
			wait_room: SyncUnsafeCell::new(VecDeque::new()),
		}
	}

	unsafe fn wait(&self) {
		let wq = &mut *self.wait_room.get();
		Task::curr_wait_in(wq);
		assert!(is_int_enabled());
		Scheduler::yield_cpu();
	}

	unsafe fn wakeup_one(&self) {
		let wq = &mut *self.wait_room.get();
		if let Some(t) = wq.pop_front() {
			t.get_task_ref_mut().wakeup();
		}
	}

	pub unsafe fn get_pool_mut(&self) -> &mut T {
		&mut (*self.resource_pool.get())
	}
}

// like the spinning one, but sleep; perhaps consider reusing code..
impl<T, E> Semaphore<T, E> for SleepSemaphore<T>
where
	T: ResourceMan<E>,
	E: Copy + Clone,
{
	fn p(&self) -> Option<E> {
		L2_GUARD.lock();
		unsafe { return self.p_unguarded() };
	}
	fn v(&self, e: E) {
		L2_GUARD.lock();
		unsafe { self.v_unguarded(e) };
	}
	unsafe fn p_unguarded(&self) -> Option<E> {
		let mut c: u64;
		loop {
			c = self.sema.load(Ordering::Relaxed);
			if c == 0 {
				unsafe { self.wait() };
				continue;
			}

			let r = self
				.sema
				.compare_exchange(c, c - 1, Ordering::Acquire, Ordering::Relaxed);
			match r {
				Ok(_) => break,
				Err(_) => continue,
			}
		}

		let thing = (&mut *self.resource_pool.get()).get_resource();
		return thing;
	}
	unsafe fn v_unguarded(&self, e: E) {
		(&mut *self.resource_pool.get()).insert_resource(e);
		let _ = self.sema.fetch_add(1, Ordering::SeqCst);
		self.wakeup_one();
	}
	fn is_empty(&self) -> bool {
		todo!()
	}
	fn is_full(&self) -> bool {
		todo!()
	}
}
