pub mod pic_8259;
pub mod pit;
use crate::io::*;
use core::arch::asm;
use core::slice;

// number of entries in IDT
pub const IDT_CAPACITY: usize = 256;
// size of interrupt handler wrapper routine (vector)
pub const VECTOR_SIZE: u64 = 16;
extern "C" {
	fn vectors_start();
	fn idt();
	fn idt_descr();
}

// x86_64 gate descriptor (idt entry) format
// [0 :15]  addr[0:15]
// [16:31]  segment selector: must point to valid code segment in GDT
// [32:39]  ist: index into Interrupt Stack Table; only lower 3 bit used,
//          other bits are reserved to 0
// [40:47]  attrs: attributes of the call gate:
//          [0:3]    - Gate Type: 0xe for interrupt and 0xf for trap
//          [ 4 ]    - Res0
//          [5:6]    - DPL: allowed privilege levels (via INT)
//          [ 7 ]    - Present (Valid)
// [48:63] - addr[16:31]
// [64:95] - addr[32:63]
#[repr(C)]
pub struct GateDescriptor64 {
	pub offset_1: u16,
	pub selector: u16,
	pub ist: u8,
	pub attrs: u8,
	pub offset_2: u16,
	pub offset_3: u32,
	res0: u32, // [96:127]
}

// TODO expand interface for idt entry, if needed.
impl GateDescriptor64 {
	#[inline(always)]
	pub fn set_offset(&mut self, offset: u64) {
		self.offset_1 = (offset & 0xffff) as u16;
		self.offset_2 = ((offset & 0xffff0000) >> 16) as u16;
		self.offset_3 = ((offset & 0xffffffff00000000) >> 32) as u32;
	}
	/// selector = 0; present; type = interrupt;
	pub fn set_default_interrupt(&mut self, offset: u64) {
		self.set_offset(offset);
		self.selector = 0x8 * 2;
		self.attrs = 0x8e;
		self.ist = 0;
		self.res0 = 0;
	}
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
extern "C" fn interrupt_gate(_slot: u16) {
	interrupt_disable();
	// NOTE: the interrupt handler should NEVER block on a lock; in this case
	// the CGA screen is protected by a spinlock. The lock holder will never be
	// able to release the lock if the interrupt handler blocks on it. Try
	// spamming the keyboard with the following line of code uncommented: it
	// will deadlock!
	// println!("interrupt received 0x{:x}", slot);
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

#[inline(always)]
fn _idt_init() {
	println!("[init] idt: vectors_start: 0x{:x}", vectors_start as usize);

	let gate_descriptors: &mut [GateDescriptor64] =
		unsafe { slice::from_raw_parts_mut(idt as *mut GateDescriptor64, 256) };

	// write to idt
	for i in 0..IDT_CAPACITY {
		let offset: u64 = vectors_start as u64 + (i as u64 * VECTOR_SIZE);
		gate_descriptors[i].set_default_interrupt(offset);
	}
	// set idtr
	unsafe { asm! ("lidt [{}]", in(reg) idt_descr) }
}

pub fn init() {
	// init idt
	_idt_init();
	// init pic
	pic_8259::_init();
}
