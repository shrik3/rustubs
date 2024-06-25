//! I/O with keyboard, cga screen and serial

use crate::machine::cgascr::CGAScreen;
use crate::machine::key::Key;
use crate::machine::keyctrl::KEY_BUFFER;
use crate::machine::serial::SerialWritter;
use crate::proc::task::Task;
use core::cell::SyncUnsafeCell;
use core::fmt;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
	pub static ref CGASCREEN_GLOBAL: Mutex<CGAScreen> = Mutex::new(CGAScreen::new());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("[{}] {}", Task::current().unwrap().pid, info);
	loop {}
}

/// the global serial writer, this is not synchronized. Used for debugging
/// where locking is not available
pub static SERIAL_GLOBAL: SyncUnsafeCell<SerialWritter> =
	SyncUnsafeCell::new(SerialWritter::new(0x3f8));

/// CGA screen print, synchronized. NEVER use in prologue
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}
pub(crate) use print;

/// CGA screen println, synchronized. NEVER use in prologue
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
pub(crate) use println;

/// serial (0x3f8 for qemu) print, not synchronized. can use in prologue
#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => ($crate::io::_serial_print(format_args!($($arg)*)));
}
pub(crate) use sprint;

#[macro_export]
/// serial (0x3f8 for qemu) println, not synchronized. can use in prologue
macro_rules! sprintln{
    () => ($crate::sprint!("\n"));
    ($($arg:tt)*) => (sprint!("{}\n", format_args!($($arg)*)));
}
pub(crate) use sprintln;

pub fn read_key() -> Key {
	use crate::proc::sync::semaphore::Semaphore;
	KEY_BUFFER.p().unwrap()
}

pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	CGASCREEN_GLOBAL.lock().write_fmt(args).unwrap();
}

pub fn _serial_print(args: fmt::Arguments) {
	use core::fmt::Write;
	unsafe {
		(*SERIAL_GLOBAL.get()).write_fmt(args).unwrap();
	}
}

/// [clear_screen] removes the content but doesn't reset the cursor
pub fn clear_screen() {
	CGASCREEN_GLOBAL.lock().clear();
}

/// [reset_screen] also resets the cursor
pub fn reset_screen() {
	CGASCREEN_GLOBAL.lock().reset();
}

pub fn back_space() {
	CGASCREEN_GLOBAL.lock().backspace();
}

pub fn print_help(s: &str, attr: u8) {
	CGASCREEN_GLOBAL.lock().print_at_bottom(s, attr);
}

pub fn set_attr(attr: u8) {
	CGASCREEN_GLOBAL.lock().setattr(attr);
}

pub fn print_welcome() {
	println!("--RuStuBs--");
	println!("    _._     _,-'\"\"`-._     ~Meow");
	println!("   (,-.`._,'(       |\\`-/|");
	println!("       `-.-' \\ )-`( , o o)");
	println!("             `-    \\`_`\"'-");
}
