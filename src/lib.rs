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
use crate::machine::key::Modifiers;
use arch::x86_64::interrupt;
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;
use core::panic::PanicInfo;
use defs::*;
use machine::cgascr::CGAScreen;
use machine::multiboot;

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
	let mbi = multiboot::get_mb_info().expect("bad multiboot info flags");
	let mem = unsafe { mbi.get_mem() }.unwrap();
	println!(
		"[init] available memory: lower {:#X} KiB, upper:{:#X} KiB",
		mem.lower(),
		mem.upper()
	);
	mm::init();
	interrupt::init();
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	interrupt::interrupt_enable();

	println!(
		"[init] kernel mapped @ {:#X} - {:#X}",
		vmap_kernel_start(),
		vmap_kernel_end(),
	);
	println!(
		"[init] BSS mapped    @ {:#X} - {:#X}",
		bss_start(),
		bss_end()
	);

	// io::print_welcome();

	// busy loop query keyboard
	loop {
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"key: {:?}", k}
		}
	}
}
