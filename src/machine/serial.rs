//! serial output through port 3f8 (qemu), stateless, non-blocking, non-locking,
//! not thread safe.

use crate::machine::device_io::IOPort;
use core::{fmt, str};
pub struct SerialWritter {
	port: IOPort,
}

impl SerialWritter {
	pub const fn new(port: u16) -> Self { Self { port: IOPort::new(port) } }

	pub fn putchar(&self, ch: char) { self.port.outb(ch as u8); }

	pub fn print(&self, s: &str) {
		for c in s.bytes() {
			self.putchar(c as char);
		}
	}
}

impl fmt::Write for SerialWritter {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.print(s);
		Ok(())
	}
}
