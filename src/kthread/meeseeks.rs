//! a noisy dummy task to test the system
use crate::arch::x86_64::misc::delay;
use crate::io::*;
use crate::kthread::KThread;
use crate::proc::task::Task;

pub struct Meeseeks {}

impl KThread for Meeseeks {
	#[no_mangle]
	extern "C" fn entry() -> ! {
		let t = Task::current().unwrap();
		sprintln!("I'm Mr.Meeseeks {}, look at me~", t.pid);
		loop {
			let t = Task::current().unwrap();
			sprintln!("I'm {}", t.pid);
			for _i in 0..1000000 {
				delay();
			}
		}
	}
}
