use crate::machine::cgascr::CGAScreen;
use crate::machine::keyctrl::KeyboardController;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
// TODO I want my own locking primitive for practice, instead of stock spin lock
lazy_static! {
	pub static ref CGASCREEN_GLOBAL: Mutex<CGAScreen> = Mutex::new(CGAScreen::new());
	pub static ref KBCTL_GLOBAL: Mutex<KeyboardController> = Mutex::new(KeyboardController::new());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}
pub(crate) use print;

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
pub(crate) use println;

pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	CGASCREEN_GLOBAL.lock().write_fmt(args).unwrap();
}

pub fn clear_screen() {
	CGASCREEN_GLOBAL.lock().clear();
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
