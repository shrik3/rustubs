// For now, the PIC is stateless, i.e. we don'e need a struct for it.
// Perhaps I need a Mutex handle later...
use crate::arch::x86_64::io_port::*;
const IMR1: u16 = 0x21;
const IMR2: u16 = 0xa1;
const CTRL1: u16 = 0x20;
const CTRL2: u16 = 0xa0;
const PIC_VECTOR_OFFSET: u8 = 0x20;

pub struct PicDeviceInt;
impl PicDeviceInt {
	pub const TIMER: u8 = 0;
	pub const KEYBOARD: u8 = 1;
}

// code and text from: https://wiki.osdev.org/8259_PIC#Code_Examples
// reprogram the PIC; _init must be called before interrupt is enabled.
// TODO: turn pic into a singleton struct
// TODO: replace these hardcoded ....
// 0x20: PIC1 COMMAND (MASTER)
// 0x21: PIC1 DATA
// 0xA0: PIC2 COMMAND (SLAVE)
// 0xA1: PIC2 DATA
pub fn init() {
	// ICW1_ICW4 | ICW1_INIT
	// start init sequence in cascade mode
	outb(CTRL1, 0x11);
	outb(CTRL2, 0x11);
	// ICW2: MASTER PIC vector offset = 0x20
	outb(IMR1, PIC_VECTOR_OFFSET);
	// ICW2: SLAVE PIC vector offset = 0x28
	outb(IMR2, PIC_VECTOR_OFFSET + 8);
	// ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
	outb(IMR1, 0x04);
	// ICW3: tell Slave PIC its cascade identity (0000 0010)
	outb(IMR2, 0x02);
	// ICW4: 8086 mode | auto (normal) EOI
	outb(IMR1, 0x03);
	outb(IMR2, 0x03);
	// set masks
	outb(IMR1, 0xfb);
	outb(IMR2, 0xff);
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
