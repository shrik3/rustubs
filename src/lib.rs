#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
#![no_main]
mod arch;
mod io;
mod machine;
use core::panic::PanicInfo;
use machine::cgascr::CGAScreen;
use machine::interrupt;
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;

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
	println!("it works!");
	
	// testing interrupt/PIC
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	interrupt::interrupt_enable();	
	loop {}
}
