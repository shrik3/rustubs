#![no_std]
#![no_main]
mod arch;
mod machine;
use core::panic::PanicInfo;
use machine::cgascr::CGAScreen;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
    let scr = CGAScreen::new(80,25);
    scr.test();
    loop {}
}
