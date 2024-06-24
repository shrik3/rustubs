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
pub static EPILOGUE_QUEUE: L3SyncCell<L2SyncQueue<EpilogueEntrant>> =
	L3SyncCell::new(L2SyncQueue::new());
/// indicates whether a task is running in L2. Maybe make it L3SyncCell as well.
static L2_AVAILABLE: AtomicBool = AtomicBool::new(true);
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

/// MUST NOT CALL THIS WHEN INT IS DISABLED
#[allow(non_snake_case)]
#[inline(always)]
pub fn SPIN_ENTER_L2() {
	let mut retry_count: u32 = 0;
	loop {
		// this for debugging only
		if retry_count > 1000 {
			panic!("can't enter L2, probably live lock")
		}
		retry_count += 1;
		// looks like a tas lock?
		let r = L2_AVAILABLE.load(Ordering::Relaxed);
		if !r {
			continue;
		}
		let cmpxhg =
			L2_AVAILABLE.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
		if let Ok(true) = cmpxhg {
			return;
		}
	}
}

/// also clear the epilogue queue before really leaving.
#[inline(always)]
#[allow(non_snake_case)]
pub fn LEAVE_L2_CLEAR_QUEUE() {
	todo!();
}

/// L3GetRef provides unsafe function `l3_get_ref` and `l3_get_ref_mut`, they
/// can only be safely used in level 3
pub trait L3GetRef<T> {
	unsafe fn l3_get_ref(&self) -> &T;
	unsafe fn l3_get_ref_mut(&self) -> &mut T;
}

/// L3SyncCell is a UnsafeCell. This abstracts global mutable states that can
/// only be accessed in Level 3, i.e. kernel mode + interrupt disabled. One
/// example is the run_queue of the global scheduler. in general for global
/// mutable state we need to use interior mutability. e.g. using Mutex. However
/// this is not desired in interrupt context.
pub type L3SyncCell<T> = SyncUnsafeCell<T>;
impl<T> L3GetRef<T> for L3SyncCell<T> {
	unsafe fn l3_get_ref(&self) -> &T {
		&*self.get()
	}
	unsafe fn l3_get_ref_mut(&self) -> &mut T {
		&mut *self.get()
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
