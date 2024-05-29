use crate::defs::*;
use crate::io::*;
use crate::machine::multiboot;
pub mod pma;

use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
	pub static ref GLOBAL_PMA: Mutex<pma::PageStackAllocator> =
		Mutex::new(pma::PageStackAllocator::new());
}

pub fn init() {
	let mbi = multiboot::get_mb_info().unwrap();
	let mmapinfo = unsafe { mbi.get_mmap() }.unwrap();
	let buf_start = mmapinfo.mmap_addr;
	let buf_len = mmapinfo.mmap_length;
	let buf_end = buf_start + buf_len;
	let mut curr = buf_start as u64;
	let mut inserted = 0;
	loop {
		if curr >= buf_end as u64 {
			break;
		}
		let mblock = unsafe { &*(curr as *const multiboot::MultibootMmap) };
		curr += mblock.size as u64;
		curr += 4;
		if mblock.mtype != multiboot::MultibootMmap::MTYPE_RAM {
			continue;
		}
		if mblock.get_end() <= pmap_kernel_start() {
			continue;
		}
		// TODO early break if the array is already full
		if mblock.get_range().contains(pmap_kernel_end()) {
			let r = Range {
				addr: pmap_kernel_end(),
				len: mblock.get_end() - pmap_kernel_end(),
			};
			inserted += GLOBAL_PMA.lock().insert_range(r);
		} else {
			inserted += GLOBAL_PMA.lock().insert_range(mblock.get_range());
		}
		println!(
			"pma init: {:#X}KiB free memory, {:#X} pages inserted from block {:#X?}",
			inserted * 0x4,
			inserted,
			mblock,
		);
	}
}
