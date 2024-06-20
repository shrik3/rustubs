//! a simple loader for statically linked elf.
use crate::arch::x86_64::paging::get_root;
use crate::arch::x86_64::paging::map_vma;
use crate::black_magic;
use crate::fs;
use crate::mm::vmm::{VMArea, VMPerms, VMType};
use crate::proc::task::Task;
use alloc::string::String;
use core::ops::Range;
use core::str::FromStr;
use xmas_elf::header::HeaderPt2;
use xmas_elf::program::ProgramHeader;
use xmas_elf::ElfFile;
pub fn cat_elf(f: &fs::File) {
	let elf = ElfFile::new(f.file).unwrap();
	println!("{:?}", elf.header);
}

// this loads a file into task address space
// half baked!
// 0. find and parse elf
// 1. creates VMAs
// 2. create paging structs and copy memory if necessary
pub fn load(file: &fs::File) -> Option<u64> {
	let task = Task::current().unwrap();
	let mm = &mut task.mm;
	let elf = ElfFile::new(file.file).ok()?;
	let pt_root = get_root();

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
		if fstart >= file.file.len() || fend >= file.file.len() {
			println!("bad size");
		}
		// black magic in sight! this converts a reference to static lifetime,
		// which is UB, but I know what I'm doing here. The file backing ARE
		// static!
		let vma = VMArea {
			vm_range: Range::<u64> {
				start: h.virtual_addr,
				end: h.virtual_addr + h.mem_size,
			},
			tag: String::from_str("USER BITS").unwrap(),
			user_perms: VMPerms::all(),
			backing: VMType::FILE(unsafe { black_magic::make_static(&file.file[fstart..fend]) }),
		};
		let res = unsafe { map_vma(pt_root, &vma, true) };
		if !res {
			println!("failed to push vma: {:#X?}", &vma);
			return None;
		}
		mm.vmas.push(vma);
	}

	if !header_ok {
		println!("bad header");
		return None;
	}
	match &elf.header.pt2 {
		HeaderPt2::Header64(pt2) => return Some(pt2.entry_point),
		_ => {
			println!("bad header, not entry point");
			return None;
		}
	};
}
