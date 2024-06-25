//! the sync module defines the OOStuBS prologue/epilogue synchronization model
//! for interrupt and preemptive scheduling. Read `docs/sync_model.md` for
//! details
#![doc = include_str!("../../../docs/sync_model.md")]
pub mod bellringer;
pub mod irq;
pub use irq::*;
pub mod semaphore;
use alloc::collections::VecDeque;
use core::cell::SyncUnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
pub static EPILOGUE_QUEUE: L3Sync<EpilogueQueue> = L3Sync::new(EpilogueQueue::new());
/// indicates whether a task is running in L2. Maybe make it L3SyncCell as well.
static L2_AVAILABLE: AtomicBool = AtomicBool::new(true);

type EpilogueQueue = L2SyncQueue<EpilogueEntrant>;

/// the synchronized queue for Level 2 epilogues
pub struct L2SyncQueue<T> {
	pub queue: VecDeque<T>,
}

impl<T> L2SyncQueue<T> {
	pub const fn new() -> Self {
		Self { queue: VecDeque::new() }
	}
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn IS_L2_AVAILABLE() -> bool {
	return L2_AVAILABLE.load(Ordering::Relaxed);
}

#[allow(non_snake_case)]
#[inline(always)]
pub fn ENTER_L2() {
	let r = L2_AVAILABLE.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
	assert_eq!(r, Ok(true));
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn LEAVE_L2() {
	let r = L2_AVAILABLE.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed);
	assert_eq!(r, Ok(false));
}

/// also clear the epilogue queue before really leaving.
#[inline(always)]
#[allow(non_snake_case)]
pub fn LEAVE_L2_CLEAR_QUEUE() {
	todo!();
}

/// L3Sync is like RefCell, that has runtime borrow checking, instead of
/// counting the reference numbers, we check that the interrupt must be disabled
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
		assert!(
			!is_int_enabled(),
			"trying to get a ref to L3 synced object with interrupt enabled"
		);
		unsafe { &*self.data.get() }
	}
	/// get a mutable reference to the protected data. will panic if called with
	/// interrupt enabled
	pub fn l3_get_ref_mut(&self) -> &mut T {
		assert!(
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

/// a handy function to wrap a (brief) block with `irq_save` and `irq_restore`
/// this must be used with caution. See docs/sync_model for how this work.
#[macro_export]
macro_rules! L3_CRITICAL{
	($($s:stmt;)*) => {
		{
			let irq = $crate::machine::interrupt::irq_save();
			$(
				$s
			)*
			$crate::machine::interrupt::irq_restore(irq);
		}
	}
}
pub use L3_CRITICAL;

use crate::arch::x86_64::is_int_enabled;
