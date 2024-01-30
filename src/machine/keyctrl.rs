use self::super::kbd_defs::*;
use self::super::key::*;
use crate::arch::x86_64::io_port::*;

// this is the driver for keyboard controller
// not to confuse with the keyboard module.
// The later is an abstraction
// This one serves a the HW driver

// TODO
// [functions]
// Keyboard_Controller()
// get_ascii_code()
// key_decoded()
// key_hit()
// reboot()
// set_led(char led,bool on)
// set_repeat_rate(int speed,int delay)
pub struct KeyboardController {
	code: u8,
	prefix: u8,
	gather: Key,
	leds: u8,

	// two ports for keyboard controller
	ctrl_port: u16,
	data_port: u16,
	// status register bits
}

impl KeyboardController {
	pub fn new() -> Self {
		Self {
			code: 0,
			prefix: 9,
			gather: Key::new(),
			leds: 0,
			ctrl_port: 0x64,
			data_port: 0x60,
		}
	}

	pub fn key_hit(&mut self) -> Key {
		todo!();
		// for debugging only
		let mut invalid: Key = Key::new();
		invalid.set_raw(0xff);

		let status = inb(self.ctrl_port);
		return Key::new();
		// TODO here
	}
}
