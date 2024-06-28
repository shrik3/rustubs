#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::io_port::*;

// either use the io functions directly, or via a IOPort instance.
pub struct IOPort(u16);
impl IOPort {
	pub const fn new(port: u16) -> Self { Self(port) }
	pub fn inw(&self) -> u16 { inw(self.0) }
	pub fn inb(&self) -> u8 { inb(self.0) }
	pub fn outw(&self, val: u16) { outw(self.0, val); }
	pub fn outb(&self, val: u8) { outb(self.0, val); }
}
