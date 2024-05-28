use crate::io::*;
// provide functions to parse information provided by grub multiboot
// see docs/multiboot.txt
extern "C" {
	static mb_magic: u32;
	static mb_info_addr: u32;
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug)]
pub struct MultibootMmap {
	pub size: u32,
	pub addr: u64,
	pub len: u64,
	pub mtype: u32,
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootInfo {
	pub flags: MultibootInfoFlags,
	pub mem_lower: u32,
	pub mem_upper: u32,
}

use bitflags::bitflags;
bitflags! {
	/// the MultibootInfoFlags indicate which fields are valid in the MultibootInfo
	/// atm we only need the MEM and MMAP flags for memory management info.
	#[derive(Copy, Clone, Debug)]
	pub struct MultibootInfoFlags: u32 {
		const MEM           = 1 << 0;
		const BOOT_DEVICE   = 1 << 1;
		const CMDLINE       = 1 << 2;
		const MODS          = 1 << 3;
		const SYM_TBL       = 1 << 4;
		const SHDR          = 1 << 5;
		const MMAP          = 1 << 6;
		const DRIVES        = 1 << 7;
		const CONF_TBL      = 1 << 8;
		const BL_NAME       = 1 << 9;
		const APM_TBL       = 1 << 10;
		const VBE_TBL       = 1 << 11;
		const FRAMEBUFFER   = 1 << 12;
	}
}

impl MultibootInfoFlags {
	/// only 13 bits of the MB info flags are defined. The other flag bits must
	/// be zero for the info block to be valie.
	const VALID_MASK: u32 = 0x1FFF;

	pub fn check_valid(&self) -> bool {
		return self.bits() <= Self::VALID_MASK;
	}
}

pub fn check_magic() -> bool {
	return unsafe { mb_magic == 0x2BADB002 };
}

pub fn get_mb_info() -> Option<MultibootInfo> {
	if !check_magic() {
		return None;
	}
	let mbi = unsafe { *(mb_info_addr as *mut MultibootInfo) };
	let flags = mbi.flags;
	if !flags.check_valid() {
		return None;
	}
	return Some(mbi);
}

// TODO: expand MultibootInfo struct defs if needed.

