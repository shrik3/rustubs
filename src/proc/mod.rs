//! process and synchronization model

use crate::arch::x86_64::is_int_enabled;
use crate::defs;
use sync::bellringer;
pub mod exec;
pub mod loader;
pub mod sched;
pub mod sync;
pub mod task;

/// this is an optimization: reserve spaces in sync array to avoid runtime
/// allocation inside of critical sections Note that the rust alloc collections
/// doesn't have a API like "set this vec to at least xyz capacity." so we can
/// only do a implicit `reserve` here. Meaning if this is called _after_ the the
/// queues receive elements, they will have more capacity than specified here.
/// safety: this function assmues interrupt is disabled
pub fn init() {
	assert!(!is_int_enabled());
	sched::GLOBAL_SCHEDULER
		.lock()
		.run_queue
		.reserve(defs::Limits::SCHED_RUN_QUEUE_MIN_CAP);
	bellringer::BELLRINGER
		.lock()
		.bedroom
		.reserve(defs::Limits::SEM_WAIT_QUEUE_MIN_CAP);
}
