//! bellringer puts tasks to sleep and wake them when semptepber ends
//! the bellringer is very much like a SleepSemaphore
use crate::machine::time;
use crate::proc::sync::{L3Sync, L3_CRITICAL};
use crate::proc::task::TaskId;
use alloc::collections::VecDeque;
pub struct BellRinger {
	pub bedroom: VecDeque<Sleeper>,
}
pub static BELLRINGER: L3Sync<BellRinger> = L3Sync::new(BellRinger::new());

#[derive(Copy, Clone, Debug)]
pub struct Sleeper {
	pub tid: TaskId,
	pub until: u64,
}

impl Sleeper {
	pub fn new(tid: TaskId, ns: u64) -> Self {
		Self { tid, until: time::nsec() + ns }
	}
}

impl BellRinger {
	pub const fn new() -> Self {
		Self { bedroom: VecDeque::new() }
	}

	pub fn check_in(s: Sleeper) {
		L3_CRITICAL! {
			let br =  BELLRINGER.l3_get_ref_mut();
			br.bedroom.push_back(s);
		}
	}
	/// check the sleeper queue and wake up if timer is due.
	/// We do this in timer interrupt epilogue
	pub fn check_all() {
		// there is much room for optimization here: the queue can be sorted and
		// instead of absolute time we can store the differntial. But I'll keep
		// it simple here.
		let now = time::nsec();
		L3_CRITICAL! {
		let br =  BELLRINGER.l3_get_ref_mut();
		unsafe {
			br.bedroom.retain(|x| {
				if x.until > now {
					true
				} else {
					x.tid.get_task_ref_mut().wakeup();
					false
				}
			})
		};
		} // end L3_CRITICAL
	}
}
