//! a noisy dummy task to test the system
use crate::arch::x86_64::misc::delay;
use crate::kthread::KThread;
use crate::proc::task::Task;

pub struct Meeseeks {}

impl KThread for Meeseeks {
	extern "C" fn entry() -> ! {
		let t = Task::current().unwrap();
		sprintln!("I'm Mr.Meeseeks {}, look at me~", t.pid);
		loop {
			let t = Task::current().unwrap();
			sprintln!("I'm {}", t.pid);
			t.nanosleep(2_000_000_000);
		}
	}
}
