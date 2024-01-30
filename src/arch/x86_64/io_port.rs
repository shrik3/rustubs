extern "C" {
	fn _inb(port: u16) -> u8;
	fn _inw(port: u16) -> u16;
	fn _outb(port: u16, val: u8);
	fn _outw(port: u16, val: u16);
}

// The port addr is 16-bit wide.
// wrappers for in/out[b,w]
// Also I don't feel necessary to have a IO_Port Class give how
// trivial it is
// TODO perhaps use inline asm, because the code is short

pub fn inw(p: u16) -> u16 {
	unsafe { _inw(p) }
}

pub fn inb(p: u16) -> u8 {
	unsafe { _inb(p) }
}
pub fn outb(p: u16, val: u8) {
	unsafe {
		_outb(p, val);
	}
}

pub fn outw(p: u16, val: u16) {
	unsafe { _outw(p, val) }
}
