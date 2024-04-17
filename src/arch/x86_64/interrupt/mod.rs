pub mod pic_8259;
pub mod pit;
use crate::io::*;
use core::arch::asm;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
extern "C" fn guardian(slot: u16) {
	interrupt_disable();
	println!("interrupt received {:x}", slot);
	interrupt_enable();
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
