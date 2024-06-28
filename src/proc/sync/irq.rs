use crate::proc::sync::L3Sync;
use alloc::collections::VecDeque;
pub static EPILOGUE_QUEUE: L3Sync<EpilogueQueue> =
	L3Sync::new(EpilogueQueue::new());
/// the synchronized queue for Level 2 epilogues
pub struct EpilogueQueue {
	pub queue: VecDeque<EpilogueEntrant>,
}

impl EpilogueQueue {
	pub const fn new() -> Self { Self { queue: VecDeque::new() } }
}

/// describes a device interrupt handler routine. This is used to register a
/// device driver to an interrupt line (see plugbox). Device drivers code should
/// implement the [IRQHandler] or [IRQHandlerEpilogue] trait.
#[derive(Copy, Clone)]
pub struct IRQGate {
	prologue: unsafe fn(),
	epilogue_entrant: Option<EpilogueEntrant>,
}

impl IRQGate {
	pub unsafe fn call_prologue(&self) { (self.prologue)(); }
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
	pub unsafe fn call(&self) { (self.epilogue)(); }
}

/// device driver trait that has only prologue
pub trait IRQHandler {
	/// the "hardirq" part of the irq handler, it must not be interrupted.
	/// unsafe: **must guarantee the prologue is called with irq disabled**
	/// the prologue should be short and bounded. It must not block.
	unsafe fn do_prologue();
	/// returns an IRQGate to be registered with the plugbox
	fn get_gate() -> IRQGate {
		IRQGate {
			prologue: Self::do_prologue,
			epilogue_entrant: None,
		}
	}
}

/// device driver trait with both prologue and epilogue
pub trait IRQHandlerEpilogue {
	/// same as in [IRQHandler]
	unsafe fn do_prologue();
	/// the "softirq" part of the irq handler, it allows interrupts but all
	/// epilogues must be linearized, therefore an **context swap must not
	/// happen when there is a running epilogue**.
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
