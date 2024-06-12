pub mod pic_8259;
pub mod pit;
pub mod plugbox;
use crate::arch::x86_64::arch_regs::TrapFrame;
use crate::arch::x86_64::is_int_enabled;
use crate::arch::x86_64::paging::fault;
use crate::defs::IntNumber as INT;
use crate::io::*;
use crate::machine::interrupt::plugbox::IRQ_GATE_MAP;
use core::arch::asm;
use core::slice;
// TODO use P2V for extern symbol addresses
/// number of entries in IDT
pub const IDT_CAPACITY: usize = 256;
/// 32 exceptions + 16 irqs from PIC = 48 valid interrupts
pub const IDT_VALID: usize = 48;
/// size of interrupt handler wrapper routine (vector)
pub const VECTOR_SIZE: u64 = 16;
extern "C" {
	fn vectors_start();
	fn idt();
	fn idt_descr();
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
extern "C" fn trap_gate(nr: u16, fp: u64) {
	// cpu automatically masks interrupts so we are already in L3
	if nr < 0x20 {
		// handle exception
		handle_exception(nr, fp);
	} else {
		unsafe { handle_irq(nr) };
		// handle irq
	}

	interrupt_enable();
}

#[inline]
/// handle_irq assumes the interrupt is disabled when called
unsafe fn handle_irq(nr: u16) {
	let irq_gate = match IRQ_GATE_MAP.get(&nr) {
		None => {
			panic!("no handler for irq {}", nr);
		}
		Some(g) => g,
	};
	// execute the prologue
	irq_gate.call_prologue();
	if let Some(epi) = irq_gate.get_epilogue() {
		epi.call();
	}
	// IRQ IS DISABLED BEFORE THIS POINT, HENCE IMPLICITLY L3
}

/// handles exception/faults (nr < 32);
#[inline]
fn handle_exception(nr: u16, fp: u64) {
	let frame = unsafe { &mut *(fp as *mut TrapFrame) };
	match nr {
		INT::PAGEFAULT => {
			// Pagefault
			let fault_address = fault::get_fault_addr();
			fault::page_fault_handler(frame, fault_address)
		}
		_ => {
			println!("[trap[ {:#X?}", frame);
			unsafe {
				asm!("hlt");
			}
		}
	}
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

#[inline]
/// irq_save() disables all interrupts and returns the previous state
pub fn irq_save() -> bool {
	if is_int_enabled() {
		interrupt_disable();
		return true;
	} else {
		return false;
	}
}

#[inline]
/// irq_restore only re-enable irq if was_enabled==true.
/// it will not disable irq regardless the was_enabled value. This function
/// should only be called to restore irq based on previous irq_save();
pub fn irq_restore(was_enabled: bool) {
	if was_enabled {
		interrupt_enable();
	}
}

/// initialize the idt: we reserved space for idt in assembly code (and linker
/// script) we write contents to them now. One purpose is to save binary space.
/// This also allows more flexibility for some interrupt handlers (e.g. when
/// they needs a dedicated stack...)
#[inline(always)]
fn _idt_init() {
	println!("[init] idt: vectors_start: 0x{:x}", vectors_start as usize);

	let gate_descriptors: &mut [GateDescriptor64] =
		unsafe { slice::from_raw_parts_mut(idt as *mut GateDescriptor64, 256) };

	// write to idt
	for i in 0..IDT_VALID {
		let offset: u64 = vectors_start as u64 + (i as u64 * VECTOR_SIZE);
		gate_descriptors[i].set_default_interrupt(offset);
	}
	let offset_inv: u64 = vectors_start as u64 + (IDT_VALID as u64 * VECTOR_SIZE);
	for i in IDT_VALID..IDT_CAPACITY {
		gate_descriptors[i].set_default_interrupt(offset_inv);
	}
	// set idtr
	unsafe { asm! ("lidt [{}]", in(reg) idt_descr) }
}

/// initialize the idt and [pic_8259]
pub fn init() {
	// init idt
	_idt_init();
	// init pic
	pic_8259::_init();
}
