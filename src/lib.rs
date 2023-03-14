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
    let scr = CGAScreen::new(25,80);
    // scr.show(0,0,'X',0x0f);
    // scr.show(0,79,'X',0x0f);
    // scr.show(24,0,'X',0x0f);
    // scr.show(24,79,'X',0x0f);
    // scr.test();
    scr.setpos(10, 10);
    loop {}
}
