//! exec a user program by loading the binary and replace the current address
//! space
//! TODO rework this code, this is only POC
use crate::fs;
use crate::proc::loader::load;
use crate::proc::task::Task;
use crate::{fs::*, Mem};
use core::arch::asm;
#[allow(unreachable_code)]
pub fn exec(file_name: &str) {
	let task = Task::current().unwrap();
	let mm = &mut task.mm;
	// wipe user vmas;
	// NOTE this doesn't wipe the pagetables
	mm.vmas
		.retain(|vma| vma.vm_range.start >= Mem::ID_MAP_START);

	let archive = get_archive();
	let file = fs::iter(archive).find(|f| f.hdr.name() == file_name);
	if file.is_none() {
		println!("error: no such file {}", file_name);
		return;
	}
	let f = file.unwrap();
	let entry = load(&f);
	if entry.is_none() {
		println!("failed to load elf");
		return;
	}
	println!("exec entry: {:#X}", entry.unwrap());
	go(entry.unwrap());
	return;
}

extern "C" fn go(entry: u64) {
	unsafe {
		asm!("push {}; ret", in(reg) entry);
	}
}
