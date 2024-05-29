use crate::io::*;
use lazy_static::lazy_static;
// provide functions to parse information provided by grub multiboot
// see docs/multiboot.txt
extern "C" {
	static mb_magic: u32;
	static mb_info_addr: u32;
}

lazy_static! {
	pub static ref MBOOTINFO: &'static MultibootInfo =
		unsafe { &*(mb_info_addr as *const MultibootInfo) };
}

/// this must be checked before any MB info fields are used.
pub fn check() -> bool {
	if unsafe { mb_magic != 0x2BADB002 } {
		return false;
	};
	// must check magic before checking flags
	let f = MBOOTINFO.get_flags();
	return f.check_valid();
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
pub struct MultibootInfoMmap {
	pub mmap_length: u32,
	pub mmap_addr: u32,
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootInfoMem {
	mem_lower: u32,
	mem_upper: u32,
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug)]
/// all fields MUST be acquired via unsafe getters, because the MB magic and reserved bits in flags
/// must be checked for validity before using. It does not suffice to check the corresponding
/// present bits in the getters.
/// Some fields are marked as padding because we don't need them (for now)
pub struct MultibootInfo {
	flags: MultibootInfoFlags,
	mem: MultibootInfoMem,
	_pad1: [u8; 32],
	mmap: MultibootInfoMmap,
	_pad2: [u8; 68],
}

impl MultibootInfo {
	// private function. Don't use it outside the module.
	fn get_flags(&self) -> MultibootInfoFlags {
		return self.flags;
	}

	pub unsafe fn get_mem(&self) -> Option<MultibootInfoMem> {
		if self.get_flags().contains(MultibootInfoFlags::MEM) {
			return Some(self.mem);
		} else {
			return None;
		}
	}

	pub unsafe fn get_mmap(&self) -> Option<MultibootInfoMmap> {
		if self.get_flags().contains(MultibootInfoFlags::MMAP) {
			return Some(self.mmap);
		} else {
			return None;
		}
	}
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

pub fn get_mb_info() -> Option<&'static MultibootInfo> {
	if !check() {
		return None;
	}
	return Some(&MBOOTINFO);
}

// TODO: expand MultibootInfo struct defs if needed.
