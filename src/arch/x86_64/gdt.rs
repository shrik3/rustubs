use crate::defs::P2V;
use core::arch::asm;

// these are 32 bit low-address symbols. we need to promote them to high
// address mapping.
extern "C" {
	fn gdt();
	fn gdt_80();
}

#[repr(C)]
#[repr(packed)]
struct GDTDescriptor {
	pub table_size: u16,
	pub table_addr: u64,
}

/// promote the gdt to high address
pub fn init() {
	let gdtd = unsafe { &mut *(gdt_80 as *mut GDTDescriptor) };
	// sanity check
	assert!(gdtd.table_size == 4 * 8 - 1);
	gdtd.table_addr = P2V(gdt as u64).unwrap();
	unsafe { asm!("lgdt [{}]", in (reg) P2V(gdt_80 as u64).unwrap()) }
}
