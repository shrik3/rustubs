use crate::proc::sync::L3Sync;
use alloc::collections::VecDeque;
pub static EPILOGUE_QUEUE: L3Sync<EpilogueQueue> =
	L3Sync::new(EpilogueQueue::new());
/// the synchronized queue for Level 2 epilogues
pub struct EpilogueQueue {
	pub queue: VecDeque<EpilogueEntrant>,
}

impl EpilogueQueue {
	pub const fn new() -> Self {
		Self { queue: VecDeque::new() }
	}
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
