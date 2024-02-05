// to detect available memory to initialize PMA
// https://wiki.osdev.org/Detecting_Memory_(x86)
use core::arch::asm;
use crate::io::*;

extern "C" {
	fn do_e820(start: usize) -> u32;
}

// Getting an E820 Memory Map -- osdev wiki
pub fn prob_mem_bios(){
	unsafe {
		let res = do_e820(0x8000);
		println!("res is {}", res)
	}
}
