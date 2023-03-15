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
    let mut scr = CGAScreen::new();
    scr.show_coners();
    scr.setattr(0x1f);
    scr.clear();
    scr.show_coners();

    scr.print("--RuStuBs--\n");
    scr.print("    _._     _,-'\"\"`-._     ~Meow\n");
    scr.print("   (,-.`._,'(       |\\`-/|\n");
    scr.print("       `-.-' \\ )-`( , o o)\n");
    scr.print("             `-    \\`_`\"'-\n");
    scr.print("it works!\n");
    loop {}
}
