//! kernel threads
pub mod meeseeks;
pub use meeseeks::Meeseeks;

pub trait KThread {
	/// the entry() should be `extern "C"` and `#[no_mangle]!`. I don't know
	/// for to enforce the later in a trait though....
	extern "C" fn entry() -> !;
	fn get_entry() -> u64 {
		Self::entry as u64
	}
}
