pub mod pic_8259;
pub mod pit;
use crate::io::*;
use core::arch::asm;

#[no_mangle]
extern "C" fn guardian(slot: u16) {
	println!("interrupt received {:x}", slot);
}

#[inline(always)]
pub fn interrupt_enable() {
	unsafe {
		asm!("sti");
	}
}

#[inline(always)]
pub fn interrupt_disable() {
	unsafe {
		asm!("cli");
	}
}
