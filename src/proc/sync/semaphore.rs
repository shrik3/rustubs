use crate::proc::task::{Task, TaskId};
use alloc::collections::VecDeque;
use core::sync::atomic::Ordering;
use core::{cell::SyncUnsafeCell, sync::atomic::AtomicU64};
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
}

/// wherever resoure management behind semaphore must provide get and insert
/// function. They do not need to be atomic. Normaly they only needs to be
/// wrappers for e.g. enque and deque.
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

pub struct SpinSemaphore<T> {
	reource_pool: SyncUnsafeCell<T>,
	sema: AtomicU64,
}

impl<T> SpinSemaphore<T> {
	pub const fn new(t: T) -> Self {
		Self {
			reource_pool: SyncUnsafeCell::new(t),
			sema: AtomicU64::new(0),
		}
	}
}

/// this simple implementaion is deadlock free but not starvation free
impl<T, E> Semaphore<T, E> for SpinSemaphore<T>
where
	T: ResourceMan<E>,
	E: Copy + Clone + core::fmt::Debug,
{
	// https://neosmart.net/blog/implementing-truly-safe-semaphores-in-rust/
	fn p(&self) -> Option<E> {
		let mut c: u64;
		loop {
			c = self.sema.load(Ordering::Relaxed);
			if c == 0 {
				continue; // sleeping semaphore will do otherwise here
			}

			let r = self
				.sema
				.compare_exchange(c, c - 1, Ordering::Acquire, Ordering::Relaxed);
			match r {
				Ok(_) => break,
				Err(_) => continue,
			}
		}

		let thing = unsafe { &mut *self.reource_pool.get() }.get_resource();
		return thing;
	}
	fn v(&self, e: E) {
		// it's important to enque BEFORE incrementing semaphore,
		// so that the producer end could be lock free.
		unsafe { &mut *self.reource_pool.get() }.insert_resource(e);
		// is SeqCst too strong?
		let _ = self.sema.fetch_add(1, Ordering::SeqCst);
	}
	fn is_empty(&self) -> bool {
		todo!()
	}
	fn is_full(&self) -> bool {
		todo!()
	}
}

// sleeping SpinSemaphore is toddo
// pub struct SleepSemaphore<T> {
// 	reource_pool: SyncUnsafeCell<T>,
// 	sema: AtomicU64,
// 	wait_room: VecDeque<TaskId>,
// }
