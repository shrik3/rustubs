pub mod pic_8259;
pub mod pit;
use crate::io::*;
use core::arch::asm;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
extern "C" fn interrupt_gate(slot: u16) {
	interrupt_disable();
	// NOTE: the interrupt handler should NEVER block on a lock; in this case
	// the CGA screen is protected by a spinlock. The lock holder will never be
	// able to release the lock if the interrupt handler blocks on it. Try
	// spamming the keyboard with the following line of code uncommented: it
	// will deadlock!
	// println!("interrupt received {:x}", slot);
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
