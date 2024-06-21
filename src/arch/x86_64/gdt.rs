//! the x86 gdt struct is so obscure and it's not worth the lines of code to
//! write proper high level representaion. Also since it only needs to be
//! written to once or twice, I'll live with the hard coded stuffs in this mod.
//! You need to suffer all this pain exists because intel/amd doens't want to
//! ditch segmentation due to backward compatibility. THIS REALLY SUCKS.
use crate::defs::P2V;
use core::arch::asm;

// these are 32 bit low-address symbols. we need to promote them to high
// address mapping.
extern "C" {
	fn gdt();
	fn gdt_80();
}

/// gdtd describes the  gdt, don't be confused, gdtd is not a gdt entdy (segment
/// descriptor)
#[repr(C)]
#[repr(packed)]
struct GDTDescriptor {
	pub table_size: u16,
	pub table_addr: u64,
}

/// promote the gdt to high address. unsafe: only call this before dropping low
/// memory mapping. and only call this once
pub unsafe fn init() {
	let gdtd = unsafe { &mut *(gdt_80 as *mut GDTDescriptor) };
	// sanity check
	assert!(gdtd.table_size == 5 * 8 - 1);
	gdtd.table_addr = P2V(gdt as u64).unwrap();
	unsafe { asm!("lgdt [{}]", in (reg) P2V(gdt_80 as u64).unwrap()) }
}
