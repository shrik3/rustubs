//! read input buffer and print
use crate::arch::x86_64::misc::delay;
use crate::io::read_key;
use crate::kthread::KThread;
use crate::proc::task::Task;

pub struct Echo {}

impl KThread for Echo {
	extern "C" fn entry() -> ! {
		let t = Task::current().unwrap();
		println!("[PID {}] WAITING FOR INPUT", t.pid);
		loop {
			let k = read_key();
			print!("{}", k.asc as char);
		}
	}
}
