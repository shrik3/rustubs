use super::misc::delay;
use core::arch::asm;

#[inline(always)]
pub fn inw(p: u16) -> u16 {
	let result: u16;
	unsafe {
		asm!("in ax, dx",
			in("dx") p,
			out("ax") result
		)
	}
	delay();
	result
}

#[inline(always)]
pub fn inb(p: u16) -> u8 {
	let result: u8;
	unsafe {
		asm!("in al, dx",
			in("dx") p,
			out("al") result
		)
	}
	delay();
	result
}

#[inline(always)]
pub fn outb(p: u16, val: u8) {
	unsafe {
		asm!("out dx, al",
			in("dx") p,
			in("al") val,
		)
	}
	delay();
}

#[inline(always)]
pub fn outw(p: u16, val: u16) {
	unsafe {
		asm!("out dx, ax",
			in("dx") p,
			in("ax") val,
		)
	}
	delay();
}
