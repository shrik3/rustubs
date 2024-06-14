//! kernel threads
pub mod meeseeks;
pub use meeseeks::Meeseeks;

pub mod echo;
pub use echo::Echo;

pub mod lazy;
pub use lazy::Lazy;

pub trait KThread {
	extern "C" fn entry() -> !;
	fn get_entry() -> u64 {
		Self::entry as u64
	}
}

pub struct Idle {}
impl KThread for Idle {
	extern "C" fn entry() -> ! {
		use core::arch::asm;
		loop {
			unsafe { asm!("sti; hlt") };
		}
	}
}
