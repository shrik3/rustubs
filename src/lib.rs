#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
#![no_main]
#![feature(const_option)]
mod arch;
mod defs;
mod ds;
mod io;
mod machine;
mod mm;
mod proc;
extern crate alloc;
use alloc::vec::Vec;
use arch::x86_64::interrupt;
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;
use core::panic::PanicInfo;
use defs::*;
use machine::cgascr::CGAScreen;
use machine::key::Modifiers;
use machine::multiboot;
use machine::serial::Serial;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {}
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
	// init code
	io::set_attr(0x1f);
	io::clear_screen();
	assert!(multiboot::check(), "bad multiboot info from grub!");
	// check mbi now. This will be later used to initilize the allocator
	let _mbi = multiboot::get_mb_info().expect("bad multiboot info flags");
	// initialize the idt and re-program the pic. Must do this before enabling irq
	// also must initialize the idt before mm, because the later may trigger page faults, which is
	// fatal and we want to catch them during system initilization.
	interrupt::init();
	mm::init();

	println!(
		"[init] kernel mapped @ {:#X} - {:#X}",
		unsafe { vmap_kernel_start() },
		unsafe { vmap_kernel_end() },
	);
	println!(
		"[init] BSS mapped    @ {:#X} - {:#X}",
		bss_start(),
		bss_end()
	);
	// busy loop query keyboard
	interrupt::interrupt_enable();
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	let mut test_vec = Vec::<&str>::new();
	test_vec.push("hello ");
	test_vec.push("world");
	for s in test_vec.iter() {
		println!("{s}");
	}
	Serial::print("hello from serial");
	loop {
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"key: {:?}", k}
		}
	}
	// test heap
}

pub unsafe fn _test_pf() {
	// try a page fault
	use core::arch::asm;
	use core::slice;
	let name_buf = slice::from_raw_parts_mut(0xffffffffffff0000 as *mut u64, 10);
	asm!("mov [rdi], rax", in("rdi") name_buf.as_mut_ptr());
}
