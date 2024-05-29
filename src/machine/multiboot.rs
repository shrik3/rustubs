use crate::io::*;
use core::fmt;
use core::mem::size_of;
use lazy_static::lazy_static;
// provide functions to parse information provided by grub multiboot
// see docs/multiboot.txt
extern "C" {
	static mb_magic: u32;
	static mb_info_addr: u32;
}

lazy_static! {
	/// reference to the multiboot info blob provided by the bootloader. This
	/// reference should be acquired via the get_mb_info() function, otherwise
	/// MUST manually call the check() function before using.
	pub static ref MBOOTINFO: &'static MultibootInfo =
		unsafe { &*(mb_info_addr as *const MultibootInfo) };
}

pub fn get_mb_info() -> Option<&'static MultibootInfo> {
	if !check() {
		return None;
	}
	return Some(&MBOOTINFO);
}

/// this must be called before any MB info fields are used: the mb_magic should
/// be correctly set and all reserved bits in mbinfo flags should be 0.
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
/// describes a a physical memory block.
pub struct MultibootMmap {
	pub size: u32,
	pub addr: u64,
	pub len: u64,
	pub mtype: u32,
}

/// defs of memory types in the multiboot's struct
impl MultibootMmap {
	/// avaialble ram
	pub const MTYPE_RAM: u32 = 1;
	/// reserved ram
	pub const MTYPE_RAM_RES: u32 = 2;
	/// usable memory holding ACPI info
	pub const MTYPE_ACPI: u32 = 3;
	/// WHAT IS THIS 4???
	pub const MTYPE_RAM_NVS: u32 = 4;
	/// defective RAM
	pub const MTYPE_RAM_DEFECT: u32 = 5;
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
/// present in MultibootInfo struct, if the corresponding flag is set. This describes the mmap
/// buffer (do not confuse with the mmap struct itself)
pub struct MultibootInfoMmap {
	pub mmap_length: u32,
	pub mmap_addr: u32,
}

/// example code to query physical memory maps: traverse the mmap structs
/// provided by bootloader.
pub fn _test_mmap() {
	let mmapinfo = unsafe { MBOOTINFO.get_mmap() }.unwrap();
	let buf_start = mmapinfo.mmap_addr;
	let buf_len = mmapinfo.mmap_length;
	let buf_end = buf_start + buf_len;
	let mut curr = buf_start as u64;
	loop {
		if curr >= buf_end as u64 {
			break;
		}
		let mblock = unsafe { &*(curr as *const MultibootMmap) };
		curr += mblock.size as u64;
		curr += 4; // mmap.size does not include the the size itself
		println!("mem block {:#X?}", mblock);
	}
}

#[repr(C)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
/// describes amount of lower and upper memory. Lower memory starts from 0,
/// maximum 640 Kib. Upper memory starts from 1MiB, size maximum is addr of the
/// _first_ upper memory hole minus 1MiB (not guaranteed)
/// Both sizes have 1KiB unit.
pub struct MultibootInfoMem {
	pub mem_lower: u32,
	pub mem_upper: u32,
}

/// the packed members needs getter, because direct access with reference may be unaligned.
/// In this case the MultibootInfoMem members are aligned to 4 bytes (u32) which should be fine,
/// but the compiler doesn't agree ... pffff it needs to be smarter
impl MultibootInfoMem {
	pub fn lower(&self) -> u32 {
		self.mem_lower
	}
	pub fn upper(&self) -> u32 {
		self.mem_upper
	}
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

// TODO: expand MultibootInfo struct defs if needed.
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
	/// be zero for the info block to be valid.
	const VALID_MASK: u32 = 0x1FFF;

	pub fn check_valid(&self) -> bool {
		return self.bits() <= Self::VALID_MASK;
	}
}

impl fmt::Debug for MultibootMmap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let addr = self.addr;
		let len = self.len;
		let mtype = self.mtype;
		write!(
			f,
			"[{}] @ {:#X} + {:#X}",
			match mtype {
				MultibootMmap::MTYPE_RAM => "GOOD",
				MultibootMmap::MTYPE_RAM_RES => "RESV",
				MultibootMmap::MTYPE_ACPI => "ACPI",
				MultibootMmap::MTYPE_RAM_NVS => "NVS ",
				MultibootMmap::MTYPE_RAM_DEFECT => "BAD ",
				_ => "UNKN",
			},
			addr,
			len,
		)
	}
}
