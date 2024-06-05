use crate::machine::device_io::IOPort;
use core::{fmt, str};
/// serial output through port 3f8 (qemu), stateless, not thread safe.
pub struct Serial {}
impl Serial {
	const PORT: IOPort = IOPort::new(0x3f8);
	pub fn putchar(ch: char) {
		Self::PORT.outb(ch as u8);
	}

	pub fn print(s: &str) {
		for c in s.bytes() {
			Self::putchar(c as char);
		}
	}
}
