// x86 programmable interrupt timer
// TODO this is a device, should not live under interrupt module
// TODO there should be an machine level timer abstraction
use crate::io::*;
use crate::machine::device_io::IOPort;
use crate::proc::sched::SET_NEED_RESCHEDULE;
// use crate::proc::sync::IRQHandler;
use crate::proc::sync::IRQHandlerEpilogue;
use crate::proc::task::Task;
// use crate::proc::sched::
pub struct PIT {}

impl PIT {
	const CTRL_PORT: IOPort = IOPort::new(0x43);
	const DATA_PORT: IOPort = IOPort::new(0x40);
	const PIT_BASE_FREQ: u64 = 1193182;
	// 1193182 Hz is roughly 838 ns
	const PIT_BASE_NS: u64 = 838;
	// max is around 54918 us (54 ms)
	pub fn set_interval(us: u64) -> u64 {
		let mut divider = (us * 1000 + Self::PIT_BASE_NS / 2) / Self::PIT_BASE_NS;
		if divider == 0 {
			panic!("how on earth can you make a zero divider?")
		}
		if divider >= 65535 {
			divider = 65535;
		}
		// TODO 65536 actually translates to 0
		Self::CTRL_PORT.outb(0x34);
		Self::DATA_PORT.outb((divider & 0xff) as u8);
		Self::DATA_PORT.outb(((divider & 0xff00) >> 8) as u8);
		return divider * Self::PIT_BASE_NS;
	}
}

impl IRQHandlerEpilogue for PIT {
	unsafe fn do_prologue() {
		// half measure: we can't set the resschedule flag when the first
		// task is not yet running i.e. before kickoff(). We need an aditional
		// check here to see if there is a valid task struct on the kernel stack;
		let _task = Task::current();
		if _task.is_none() {
			return;
		}
		let _ = SET_NEED_RESCHEDULE();
	}
	unsafe fn do_epilogue() {}
}
