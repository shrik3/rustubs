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
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;
use core::panic::PanicInfo;
use machine::cgascr::CGAScreen;
use machine::interrupt;

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
	pic_8259::init();
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	interrupt::interrupt_enable();
	io::set_attr(0x1f);
	io::clear();
	println!("--RuStuBs--");
	println!("    _._     _,-'\"\"`-._     ~Meow");
	println!("   (,-.`._,'(       |\\`-/|");
	println!("       `-.-' \\ )-`( , o o)");
	println!("             `-    \\`_`\"'-");
	//
	// busy loop query keyboard
	let mut framemap = mm::pma::FMap::new();
	framemap.init();
	println!("Bitmap starting from : {:p}", framemap.bm.as_ptr());
	println!("Skip first {} bytes", framemap.skip_byte);

	use crate::machine::device_io::IOPort;
	loop {
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"caught key: {:?}", k}
		}
	}
}
