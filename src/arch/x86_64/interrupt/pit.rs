// x86 programmable interrupt timer
// TODO there should be an machine level timer abstraction
use crate::machine::device_io::IOPort;
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
