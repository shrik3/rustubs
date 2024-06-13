//! kernel threads
pub mod meeseeks;
pub use meeseeks::Meeseeks;

pub mod echo;
pub use echo::Echo;

pub trait KThread {
	extern "C" fn entry() -> !;
	fn get_entry() -> u64 {
		Self::entry as u64
	}
}
