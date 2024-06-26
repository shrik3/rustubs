use crate::defs::HWDefs::*;
use crate::io::*;
use crate::ExternSyms::{idt, idt_descr, vectors_start};
use core::arch::asm;
use core::slice;

/// initialize the idt: we reserved space for idt in assembly code (and linker
/// script) we write contents to them now. One purpose is to save binary space.
/// This also allows more flexibility for some interrupt handlers (e.g. when
/// they needs a dedicated stack...)
#[inline(always)]
pub fn init() {
	println!("[init] idt: vectors_start: 0x{:x}", vectors_start as usize);

	let gate_descriptors: &mut [GateDescriptor64] = unsafe {
		slice::from_raw_parts_mut(idt as *mut GateDescriptor64, IDT_CAPACITY)
	};

	// write to idt
	for i in 0..IDT_VALID {
		let offset = vectors_start as usize + (i * VECTOR_SIZE);
		gate_descriptors[i].set_default_interrupt(offset as u64);
	}
	let offset_inv: usize = vectors_start as usize + (IDT_VALID * VECTOR_SIZE);
	for i in IDT_VALID..IDT_CAPACITY {
		gate_descriptors[i].set_default_interrupt(offset_inv as u64);
	}
	// set idtr
	unsafe { asm! ("lidt [{}]", in(reg) idt_descr) }
}

/// ```text
/// <pre>
/// [0 :15]  addr[0:15]
/// [16:31]  segment selector: must point to valid code segment in GDT
/// [32:39]  ist: index into Interrupt Stack Table; only lower 3 bit used,
///          other bits are reserved to 0
/// [40:47]  attrs: attributes of the call gate:
///          [0:3]    - Gate Type: 0xe for interrupt and 0xf for trap
///          [ 4 ]    - Res0
///          [5:6]    - DPL: allowed privilege levels (via INT)
///          [ 7 ]    - Present (Valid)
/// [48:63] - addr[16:31]
/// [64:95] - addr[32:63]
/// ```
#[repr(C)]
#[repr(packed)]
struct GateDescriptor64 {
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
	fn set_offset(&mut self, offset: u64) {
		self.offset_1 = (offset & 0xffff) as u16;
		self.offset_2 = ((offset & 0xffff0000) >> 16) as u16;
		self.offset_3 = ((offset & 0xffffffff00000000) >> 32) as u32;
	}
	/// selector = 0; present; type = interrupt;
	fn set_default_interrupt(&mut self, offset: u64) {
		self.set_offset(offset);
		self.selector = 0x8 * 2;
		self.attrs = 0x8e;
		self.ist = 0;
		self.res0 = 0;
	}
}
