//! read input buffer and print
use crate::kthread::KThread;
use crate::machine::time;
use crate::proc::task::Task;
/// test system bellringer
pub struct Lazy {}

impl KThread for Lazy {
	fn entry() -> ! {
		let t = Task::current().unwrap();
		loop {
			sprintln!("CLOCK: {} s", time::sec());
			t.nanosleep(1_000_000_000);
		}
	}
}
