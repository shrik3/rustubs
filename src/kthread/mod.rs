//! kernel threads
pub mod meeseeks;
pub use meeseeks::Meeseeks;

pub mod echo;
pub use echo::Echo;

pub mod lazy;
pub use lazy::Lazy;

use crate::proc::sync::LEAVE_L2;

pub trait KThread {
	fn entry() -> !;
	/// the actual entry, where we also explicitly release L2
	extern "C" fn _entry() {
		LEAVE_L2();
		Self::entry();
	}
	fn get_entry() -> u64 {
		Self::_entry as u64
	}
}

pub struct Idle {}
impl KThread for Idle {
	fn entry() -> ! {
		use core::arch::asm;
		loop {
			unsafe { asm!("sti; hlt") };
		}
	}
}
