//! bellringer puts tasks to sleep and wake them when semptepber ends
//! the bellringer is very much like a SleepSemaphore
use crate::machine::time;
use crate::proc::sync::L2Sync;
use crate::proc::task::TaskId;
use alloc::collections::VecDeque;
pub struct BellRinger {
	pub bedroom: VecDeque<Sleeper>,
}
pub static BELLRINGER: L2Sync<BellRinger> = L2Sync::new(BellRinger::new());

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
		BELLRINGER.lock().bedroom.push_back(s);
	}
	/// check the sleeper queue and wake up if timer is due.
	/// this is only to be called in epilogues
	pub unsafe fn check_all() {
		// there is much room for optimization here: the queue can be sorted and
		// instead of absolute time we can store the differntial. But I'll keep
		// it simple here.
		let now = time::nsec();
		BELLRINGER.get_ref_mut_unguarded().bedroom.retain(|x| {
			if x.until > now {
				true
			} else {
				x.tid.get_task_ref_mut().wakeup();
				false
			}
		})
	}
}
