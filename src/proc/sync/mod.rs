//! the sync module defines the OOStuBS prologue/epilogue synchronization model
//! for interrupt and preemptive scheduling. Read `docs/sync_model.txt` for
//! details
pub mod bellringer;
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
		Self {
			queue: VecDeque::new(),
		}
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

#[derive(Copy, Clone)]
pub struct IRQGate {
	/// the "hardirq" part of the irq handler, it must not be interrupted.
	/// unsafe: **must guarantee the prologue is called with irq disabled**
	/// the prologue should be short and bounded. It must not block.
	prologue: unsafe fn(),
	/// the "softirq" part of the irq handler, it allows interrupts but all
	/// epilogues must be linearized, therefore an **context swap must not
	/// happen when there is a running epilogue**. For this reason , this
	/// function is unsafe. optional. If present the require_epilogue function
	/// should return true.
	/// the performance difference? the require_epilogue function?
	epilogue_entrant: Option<EpilogueEntrant>,
}

impl IRQGate {
	pub unsafe fn call_prologue(&self) {
		(self.prologue)();
	}
	/// the epilogue function should never be directly called from the IRQGate.
	/// Instead you need to get an epilogue entrant, and insert it into the
	/// epilogue queue, so that the execution of epilogues are synchronized.
	pub fn get_epilogue(&self) -> Option<EpilogueEntrant> {
		return self.epilogue_entrant;
	}
}

/// should epilogue take parameter(s)? if so, the parameters should be stored in
/// the EpilogueEntrant objects but how to make it generic in a strongly typed
/// system? Since each epilogue may require different parameter types.. passing
/// raw pointers isn't an option because memory (ownership) safety
#[derive(Copy, Clone)]
pub struct EpilogueEntrant {
	epilogue: unsafe fn(),
}

impl EpilogueEntrant {
	/// call the actuall epilogue routine. unsafe because this must be
	/// synchronized in L2
	pub unsafe fn call(&self) {
		(self.epilogue)();
	}
}

pub trait IRQHandler {
	unsafe fn do_prologue();
	fn get_gate() -> IRQGate {
		IRQGate {
			prologue: Self::do_prologue,
			epilogue_entrant: None,
		}
	}
}

pub trait IRQHandlerEpilogue {
	unsafe fn do_prologue();
	unsafe fn do_epilogue();
	fn get_gate() -> IRQGate {
		IRQGate {
			prologue: Self::do_prologue,
			epilogue_entrant: Some(EpilogueEntrant {
				epilogue: Self::do_epilogue,
			}),
		}
	}
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
			let irq = crate::machine::interrupt::irq_save();
			$(
				$s
			)*
			crate::machine::interrupt::irq_restore(irq);
		}
	}
}
pub use L3_CRITICAL;
