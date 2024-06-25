//! the x86 gdt struct is so obscure and it's not worth the lines of code to
//! write proper high level representaion. Also since it only needs to be
//! written to once or twice, I'll live with the hard coded stuffs in this mod.
//! You need to suffer all this pain exists because intel/amd doens't want to
//! ditch segmentation due to backward compatibility. THIS REALLY SUCKS.
use crate::defs::P2V;
use bit_field::BitField;
use core::mem::size_of;
use core::{arch::asm, slice::from_raw_parts_mut};

// these are 32 bit low-address symbols. we need to promote them to high
// address mapping.
extern "C" {
	fn gdt();
	// tss_desc is part of the gdt, this tag is only for convenience
	fn tss_desc();
	fn gdt_80();
	// tss0 is already reserved in high memory
	fn tss0();
}

/// promote the gdt to high address. unsafe: only call this before dropping low
/// memory mapping. and only call this once
pub unsafe fn init() {
	let gdtd = unsafe { &mut *(gdt_80 as *mut GDTDescriptor) };
	// sanity check
	assert!(gdtd.table_size == 7 * 8 - 1);
	gdtd.table_addr = P2V(gdt as u64).unwrap();
	unsafe { asm!("lgdt [{}]", in (reg) P2V(gdt_80 as u64).unwrap()) }
	// set up tss
	let tssd = from_raw_parts_mut(tss_desc as *mut u64, 2);
	let (low, high) = to_tss_desc(tss0 as u64);
	tssd[0] = low;
	tssd[1] = high;
	// load tss. Fuck you x86 why this one don't need to minus one?
	// 0x28 for the 6th entry in gdt.
	asm!("ltr {0:x}", in(reg) 0x28, options(nostack, preserves_flags));
}

pub unsafe fn set_tss_ksp(ksp: u64) {
	let tss = tss0 as *mut TaskStateSegment;
	(*tss).privilege_stack_table[0] = ksp;
}

/// gdtd describes the  gdt, don't be confused, gdtd is not a gdt entdy (segment
/// descriptor)
#[repr(C)]
#[repr(packed)]
struct GDTDescriptor {
	pub table_size: u16,
	pub table_addr: u64,
}

// takes the address of the tss segment and returns its descriptor in the
// gdt(high, low). The low desc contains only the PRESENT bit
fn to_tss_desc(tss_addr: u64) -> (u64, u64) {
	// present
	let mut low: u64 = 1 << 47;
	// base
	low.set_bits(16..40, tss_addr.get_bits(0..24));
	low.set_bits(56..64, tss_addr.get_bits(24..32));
	// limit (the `-1` in needed since the bound is inclusive)
	low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
	// type (0b1001 = available 64-bit tss)
	low.set_bits(40..44, 0b1001);
	let mut high: u64 = 0;
	high.set_bits(0..32, tss_addr.get_bits(32..64));
	(low, high)
}

// below are copied from:
// https://docs.rs/x86_64/0.15.1/src/x86_64/structures/tss.rs.html#12-23
// TODO: add attributions
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
	reserved_1: u32,
	/// The full 64-bit canonical forms of the stack pointers (RSP) for
	/// privilege levels 0-2.
	pub privilege_stack_table: [u64; 3],
	reserved_2: u64,
	/// The full 64-bit canonical forms of the interrupt stack table (IST)
	/// pointers.
	pub interrupt_stack_table: [u64; 7],
	reserved_3: u64,
	reserved_4: u16,
	/// The 16-bit offset to the I/O permission bit map from the 64-bit TSS
	/// base.
	pub iomap_base: u16,
}

impl TaskStateSegment {
	/// Creates a new TSS with zeroed privilege and interrupt stack table and an
	/// empty I/O-Permission Bitmap.
	///
	/// As we always set the TSS segment limit to
	/// `size_of::<TaskStateSegment>() - 1`, this means that `iomap_base` is
	/// initialized to `size_of::<TaskStateSegment>()`.
	#[inline]
	pub const fn new() -> TaskStateSegment {
		TaskStateSegment {
			privilege_stack_table: [0; 3],
			interrupt_stack_table: [0; 7],
			iomap_base: size_of::<TaskStateSegment>() as u16,
			reserved_1: 0,
			reserved_2: 0,
			reserved_3: 0,
			reserved_4: 0,
		}
	}
}
