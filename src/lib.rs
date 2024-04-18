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
use arch::x86_64::interrupt;
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;
use core::panic::PanicInfo;
use machine::cgascr::CGAScreen;

use crate::machine::key::Modifiers;

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
	interrupt::init();
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	interrupt::interrupt_enable();
	io::print_welcome();
	let mut framemap = mm::pma::FMap::new();
	framemap.init();
	println!("Bitmap starting from : {:p}", framemap.bm.as_ptr());
	println!("Skip first {} bytes", framemap.skip_byte);

	// busy loop query keyboard
	loop {
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"key: {:?}", k}
		}
	}
}
