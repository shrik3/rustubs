//! system level timer

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
// using PIT: this is not really accurate
// 64 bit nanosecond timer takes hundreds of years to wrap around.
pub static HW_NS: AtomicU64 = AtomicU64::new(0);
// TODO remove hardcode
pub static HW_INCREMENT: u64 = 19999708;

// call this in the timer interrupt prologue
pub fn tick() -> u64 {
	HW_NS.fetch_add(HW_INCREMENT, Ordering::Relaxed)
}

pub fn sec() -> u64 {
	let ns = HW_NS.load(Ordering::Relaxed);
	ns / 1_000_000_000
}

pub fn msec() -> u64 {
	let ns = HW_NS.load(Ordering::Relaxed);
	ns / 1_000_000
}

pub fn nsec() -> u64 {
	HW_NS.load(Ordering::Relaxed)
}
