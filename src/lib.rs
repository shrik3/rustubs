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

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {}
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
	io::set_attr(0x1f);
	io::clear();
	println!("--RuStuBs--");
	println!("    _._     _,-'\"\"`-._     ~Meow");
	println!("   (,-.`._,'(       |\\`-/|");
	println!("       `-.-' \\ )-`( , o o)");
	println!("             `-    \\`_`\"'-");
	// testing interrupt/PIC
	// pic_8259::allow(PicDeviceInt::KEYBOARD);
	// interrupt::interrupt_enable();
	//
	// busy loop query keyboard
	loop {
		// let code = io::KBCTL_GLOBAL.lock().simple_read();
		io::KBCTL_GLOBAL.lock().fetch_key();
		if let Some(k) = io::KBCTL_GLOBAL.lock().consume_key() {
			println! {"caught key: {:?}", k}
		}
	}
}
