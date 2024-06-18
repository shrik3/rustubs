//! exec a user program by loading the binary and replace the current address
//! space
//! TODO rework this code, this is only POC
use crate::fs;
use crate::mm::vmm::{VMArea, VMPerms, VMType};
use crate::proc::task::Task;
use crate::{fs::*, Mem};
use alloc::string::String;
use core::arch::asm;
use core::ops::Range;
use core::str::FromStr;
use xmas_elf::header::HeaderPt2;
use xmas_elf::program::ProgramHeader;
use xmas_elf::ElfFile;
#[allow(unreachable_code)]
pub fn exec(file_name: &str) {
	let task = Task::current().unwrap();
	let mm = &mut task.mm;
	// wipe user vmas;
	mm.vmas
		.retain(|vma| vma.vm_range.start >= Mem::ID_MAP_START);

	let archive = get_archive();
	let file = fs::iter(archive).find(|f| f.hdr.name() == file_name);
	if file.is_none() {
		println!("error: no such file {}", file_name);
		return;
	}
	// file is valid
	let f = file.unwrap();
	let elf = ElfFile::new(f.file);
	if elf.is_err() {
		println!("error: not a valid elf {}", file_name);
		return;
	}
	let elf = elf.unwrap();
	println!("starting: {}", file_name);

	let mut header_ok = true;
	for hdr in elf.program_iter() {
		// you stupid rust force people to do 20000 levels of
		// indentations of if let and match. Why can't we have nice
		// thing?
		if let ProgramHeader::Ph32(_) = hdr {
			println!("not an 64 bit elf header");
			header_ok = false;
			break;
		}
		// I know for sure now this is Ph64, I just want to unwrap it. Why the
		// heck can't I do it? Why the heck do I need yet another level of
		// indentation???
		let h = match hdr {
			ProgramHeader::Ph64(h) => h,
			_ => panic!(),
		};
		println!(
			"{:?} VA:{:#X}+{:#X}, FILE:{}+{:#X}",
			h.type_, h.virtual_addr, h.mem_size, h.offset, h.file_size,
		);
		if h.mem_size != h.file_size {
			println!("BSS not supported, yet");
			header_ok = false;
			break;
		}
		let fstart = h.offset as usize;
		let fend = fstart + h.file_size as usize;
		if fstart >= f.file.len() || fend >= f.file.len() {
			println!("bad size");
		}
		let vma = VMArea {
			vm_range: Range::<u64> {
				start: h.virtual_addr,
				end: h.mem_size,
			},
			tag: String::from_str("USER BITS").unwrap(),
			user_perms: VMPerms::all(),
			backing: VMType::FILE(&f.file[fstart..fend]),
		};
		mm.vmas.push(vma);
	}
	// clean up and quit
	if !header_ok {
		return;
	}
	let entry = match &elf.header.pt2 {
		HeaderPt2::Header64(pt2) => pt2.entry_point,
		_ => {
			println!("bad header, not entry point");
			return;
		}
	};
	go(entry);
	println!("entry: {:#X}", entry);
	return;
}

extern "C" fn go(entry: u64) {
	unsafe {
		asm!("push {}; ret", in(reg) entry);
	}
}
