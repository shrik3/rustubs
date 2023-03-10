#![no_std]
#![no_main]
mod arch;
// use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//
//     loop {}
// }

#[no_mangle]
pub extern "C" fn _entry() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    unsafe {
        *vga_buffer.offset(10 as isize * 2) = 'X' as u8;
        *vga_buffer.offset(10 as isize * 2 + 1) = 0xb;
    }
    loop {}
}
