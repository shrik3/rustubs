#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
#![no_main]
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
	let mmap = unsafe { mbi.get_mmap() }.unwrap();
	println!("memory: {:#X?}", mem);
	println!("mmap (start): {:#X?}", mmap);

	multiboot::_test_mmap();
	interrupt::init();
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	interrupt::interrupt_enable();
	let mut framemap = mm::pma::FMap::new();
	framemap.init();
	println!("Bitmap starting from : {:p}", framemap.bm.as_ptr());
	println!("Skip first {} bytes", framemap.skip_byte);
	println!("system init .. done!");
	// io::print_welcome();

	// busy loop query keyboard
	loop {
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"key: {:?}", k}
		}
	}
}
