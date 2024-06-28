//! the sync module defines the OOStuBS prologue/epilogue synchronization model
//! for interrupt and preemptive scheduling. Read `docs/sync_model.md` for
//! details
#![doc = include_str!("../../../docs/sync_model.md")]
pub mod bellringer;
pub mod irq;
pub mod semaphore;
use crate::arch::x86_64::is_int_enabled;
use crate::black_magic::Empty;
use core::cell::SyncUnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
pub use irq::*;
/// indicates whether a task is running in L2. Maybe make it L3SyncCell as well.
static L2_AVAILABLE: AtomicBool = AtomicBool::new(true);
/// RAII lock guard for the global L2 flag, the u64 is not to be used.
static L2_GUARD: L2Sync<Empty> = L2Sync::new(Empty::new());

#[inline(always)]
#[allow(non_snake_case)]
pub fn IS_L2_AVAILABLE() -> bool {
	return L2_AVAILABLE.load(Ordering::Relaxed);
}

#[allow(non_snake_case)]
#[inline(always)]
pub fn ENTER_L2() {
	let r = L2_AVAILABLE.compare_exchange(
		true,
		false,
		Ordering::Relaxed,
		Ordering::Relaxed,
	);
	debug_assert_eq!(r, Ok(true));
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn LEAVE_L2() {
	let r = L2_AVAILABLE.compare_exchange(
		false,
		true,
		Ordering::Relaxed,
		Ordering::Relaxed,
	);
	debug_assert_eq!(r, Ok(false));
}

/// also clear the epilogue queue before really leaving.
#[inline(always)]
#[allow(non_snake_case)]
pub fn LEAVE_L2_CLEAR_QUEUE() {
	todo!();
}

/// RAII guard for L2Sync objects
pub struct L2Guard<'a, T: 'a> {
	lock: &'a L2Sync<T>,
	// poison is implicit (using the L2_AVAILABLE flag)
}

impl<'a, T> Deref for L2Guard<'a, T> {
	type Target = T;
	fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
}

impl<'a, T> DerefMut for L2Guard<'a, T> {
	fn deref_mut(&mut self) -> &mut T { unsafe { &mut *self.lock.data.get() } }
}

impl<'a, T> Drop for L2Guard<'a, T> {
	fn drop(&mut self) { LEAVE_L2(); }
}

/// All L2Sync objects are guaranteed to be synchronized on the epilogue level.
pub struct L2Sync<T> {
	data: SyncUnsafeCell<T>,
}

impl<T> L2Sync<T> {
	pub const fn new(data: T) -> Self {
		Self { data: SyncUnsafeCell::new(data) }
	}
	pub fn lock(&self) -> L2Guard<T> {
		ENTER_L2();
		L2Guard { lock: self }
	}

	/// This breaks synchronization, the caller is responsible of checking the
	/// global L2_AVAILABLE flag, and do other stuffs (like relaying) when
	/// epilogue level is occupied.
	pub unsafe fn get_ref_unguarded(&self) -> &T { &*self.data.get() }

	pub unsafe fn get_ref_mut_unguarded(&self) -> &mut T {
		&mut *self.data.get()
	}
}

/// L3Sync is like RefCell, instead of counting the reference numbers, we check
/// that the interrupt must be disabled. e.g. epilogue queue
///
/// TODO: implement reference counting to make sure the sync model is followed
pub struct L3Sync<T> {
	data: SyncUnsafeCell<T>,
}

impl<T> L3Sync<T> {
	pub const fn new(data: T) -> Self {
		Self { data: SyncUnsafeCell::new(data) }
	}
	/// get a readonly reference to the protected data. It should be fine to get
	/// a read only ref without masking interrupts but we haven't implemented
	/// reference counting yet so ...
	pub fn l3_get_ref(&self) -> &T {
		debug_assert!(
			!is_int_enabled(),
			"trying to get a ref to L3 synced object with interrupt enabled"
		);
		unsafe { &*self.data.get() }
	}
	/// get a mutable reference to the protected data. will panic if called with
	/// interrupt enabled
	pub fn l3_get_ref_mut(&self) -> &mut T {
		debug_assert!(
			!is_int_enabled(),
			"trying to get a mut ref to L3 synced object with interrupt enabled"
		);
		unsafe { &mut *self.data.get() }
	}
	/// get a mutable reference without checking sync/borrow conditions.
	pub unsafe fn l3_get_ref_mut_unchecked(&self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}
