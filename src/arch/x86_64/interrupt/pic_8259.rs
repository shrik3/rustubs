// For now, the PIC is stateless, i.e. we don'e need a struct for it.
// Perhaps I need a Mutex handle later...
use crate::arch::x86_64::io_port::*;
use crate::arch::x86_64::misc::*;
const IMR1: u16 = 0x21;
const IMR2: u16 = 0xa1;
const CTRL1: u16 = 0x20;
const CTRL2: u16 = 0xa0;

pub struct PicDeviceInt;
impl PicDeviceInt {
	pub const TIMER: u8 = 0;
	pub const KEYBOARD: u8 = 1;
}

// init must be called before interrupt is enabled.
// TODO: turn pic into a singleton struct
pub fn init() {
	outb(0x20, 0x11);
	outb(0xa0, 0x11);
	outb(0x21, 0x20);
	outb(0xa1, 0x28);
	outb(0x21, 0x04);
	outb(0xa1, 0x02);
	outb(0x21, 0x03);
	outb(0xa1, 0x03);
	outb(0xa1, 0xff);
	outb(0x21, 0xfb);
}

// 8-bit registers IMR1 and IMR2 registers hold interrupt masking bit 0~7 and
// 8~15; if an interrupt is masked(set 1) on the respective bit, it's disabled
pub fn allow(interrupt: u8) {
	if interrupt < 8 {
		let old = inb(IMR1);
		outb(IMR1, old & !(1 << interrupt));
	} else {
		let old = inb(IMR2);
		outb(IMR2, old & !(1 << (interrupt - 8)));
	}
}

pub fn forbid(interrupt: u8) {
	if interrupt < 8 {
		let old = inb(IMR1);
		outb(IMR1, old | (1 << interrupt));
	} else {
		let old = inb(IMR2);
		outb(IMR2, old | (1 << (interrupt - 8)));
	}
}

pub fn is_masked(interrupt: u8) -> bool {
	if interrupt < 8 {
		let val = inb(IMR1);
		val & (interrupt) != 0
	} else {
		let val = inb(IMR2);
		val & (interrupt - 8) != 0
	}
}
