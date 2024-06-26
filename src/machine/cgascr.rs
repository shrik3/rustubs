use crate::machine::device_io::*;
use crate::{arch::x86_64::misc::*, P2V};
use core::{fmt, ptr, slice, str};

// I would consider these cga parameters constant. the scroll() and clear() work
// with the assumption that the CGAScreen memory buffer is 64-bit aligned
// luckily it is. if any changes, you'd better hope the assumption still holds
// For each character, it takes 2 byte in the buffer (one for char and one for
// attribute) Therefore the MAX_COLS should be a multiple of 4 broken..
//
// TODO: clean me up
const MAX_COLS: usize = 80;
const MAX_ROWS: usize = 25;
const CGA_BUFFER_START: usize = P2V(0xb8000).unwrap() as usize;

pub struct CGAScreen {
	pub cga_mem: &'static mut [u8],
	cursor_r: usize,
	cursor_c: usize,
	attr: u8,
}

#[inline(always)]
fn cal_offset(row: usize, col: usize) -> usize {
	col + row * MAX_COLS
}

impl CGAScreen {
	const IR_PORT: IOPort = IOPort::new(0x3d4);
	const DR_PORT: IOPort = IOPort::new(0x3d5);
	pub fn new() -> Self {
		let cga = Self {
			cga_mem: unsafe {
				slice::from_raw_parts_mut(
					CGA_BUFFER_START as *mut u8,
					2 * MAX_COLS * MAX_ROWS,
				)
			},
			cursor_r: 0,
			cursor_c: 0,
			attr: 0x0f,
		};
		cga.init_cursor();
		return cga;
	}

	#[inline(always)]
	/// put a char at a position, it doesn't care about the stored
	/// cursor location.
	pub fn show(&mut self, row: usize, col: usize, c: char, attr: u8) {
		let index = cal_offset(row, col);
		self.cga_mem[index * 2] = c as u8;
		self.cga_mem[index * 2 + 1] = attr;
	}

	/// print a char at the current cursor location and update the cursor.
	/// Scroll the screen if needed. This doesn't sync the on-screen cursor
	/// because IO takes time. the print function updates the cursor location in
	/// the end.
	fn putchar(&mut self, ch: char) {
		// TODO align to next tabstop on \t ... but this will make backspace
		// trickier ..
		match ch {
			'\n' => self.newline(),
			_ => {
				self.show(self.cursor_r, self.cursor_c, ch, self.attr);
				self.advance();
			}
		}
	}

	#[inline(always)]
	pub fn backspace(&mut self) {
		if self.cursor_c == 0 && self.cursor_r == 0 {
			return;
		}
		if self.cursor_c == 0 {
			self.cursor_r -= 1;
			self.cursor_c = MAX_COLS;
		} else {
			self.cursor_c -= 1;
		}
		self.sync_cursor();
		self.show(self.cursor_r, self.cursor_c, 0 as char, self.attr);
	}

	// this assumes 1) every line is at least qword aligned and 2) width=80chars
	// move a whole line, overwrites the target line.
	// this is a helper function to scroll up/down
	fn move_line(from: usize, to: usize) {
		if !(0..MAX_ROWS).contains(&from) {
			return;
		}
		if !(0..MAX_ROWS).contains(&to) {
			return;
		}
		if from == to {
			return;
		}
		// every line is 80 chars, 160 bytes, 20 x 8 bytes
		let src: usize = CGA_BUFFER_START + from * 160;
		let dst: usize = CGA_BUFFER_START + to * 160;
		unsafe {
			ptr::copy_nonoverlapping(src as *mut u64, dst as *mut u64, 20);
		}
	}

	// clear a line but leave the attributes
	fn clearline(line: usize, attr: u8) {
		let mut base: u64 = (attr as u64) << 8;
		base += base << 16;
		base += base << 32;
		if !(0..MAX_ROWS).contains(&line) {
			return;
		}
		unsafe {
			slice::from_raw_parts_mut(
				(CGA_BUFFER_START + line * 160) as *mut u64,
				20,
			)
			.fill(base);
		}
	}

	/// advance the cga screen's internal book keeping of the cursor location by
	/// 1 char. Scroll up if a newline is required at the last line.
	fn advance(&mut self) {
		if self.cursor_c >= (MAX_COLS - 1) {
			self.newline();
		} else {
			self.cursor_c += 1;
		}
	}

	/// move cursor to the start of the next line. Scroll screen if called on
	/// the last line.
	fn newline(&mut self) {
		if self.cursor_r >= (MAX_ROWS - 1) {
			self.scroll(1);
		} else {
			self.cursor_r += 1;
		}
		self.cursor_c = 0;
	}

	pub fn scroll(&self, lines: usize) {
		if lines >= MAX_ROWS {
			self.clear();
		}

		if lines == 0 {
			return;
		}
		// jump over first n rows since they will be overwrriten anyways;
		for i in lines..MAX_ROWS {
			Self::move_line(i, i - lines);
		}
		// and clear the rest
		for i in (MAX_ROWS - lines)..MAX_ROWS {
			Self::clearline(i, self.attr);
		}
	}

	pub fn clear(&self) {
		for i in 0..MAX_ROWS {
			Self::clearline(i, self.attr);
		}
	}

	pub fn reset(&mut self) {
		self.clear();
		self.setpos(0, 0);
		self.sync_cursor();
	}

	/// the on-screen cursor position is decoupled with the system's own book
	/// keeping, i.e. we update the cursor position based on our own record, but
	/// we never read this position back.
	pub fn setpos(&mut self, row: usize, col: usize) {
		self.cursor_r = row;
		self.cursor_c = col;
	}

	fn sync_cursor(&self) {
		// io ports for instruction register and data register
		let offset = cal_offset(self.cursor_r, self.cursor_c);
		// set lower byte
		Self::IR_PORT.outb(15_u8);
		delay();
		Self::DR_PORT.outb(offset as u8);
		// set higher byte
		Self::IR_PORT.outb(14_u8);
		delay();
		Self::DR_PORT.outb((offset >> 8) as u8);
	}

	// make cursor blink (is this necessary??)
	pub fn init_cursor(&self) {
		Self::IR_PORT.outb(0x0a);
		delay();
		let mut d = Self::DR_PORT.inb();
		delay();
		d &= 0xc0;
		d |= 0xe;
		Self::DR_PORT.outb(d);
		delay();
		Self::IR_PORT.outb(0x0b);
		delay();
		let mut d = Self::DR_PORT.inb();
		d &= 0xe0;
		d |= 0xf;
		Self::DR_PORT.outb(d);
	}

	pub fn print(&mut self, s: &str) {
		for c in s.bytes() {
			self.putchar(c as char);
		}
		self.sync_cursor();
	}

	/// this is helpful for some helper text.
	/// this will clear the bottom line
	/// this will not update the cursor location.
	pub fn print_at_bottom(&mut self, s: &str, attr: u8) {
		Self::clearline(24, self.attr);
		let s = if s.len() >= MAX_COLS { &s[0..MAX_COLS] } else { s };
		// ugly!
		let orig_r = self.cursor_r;
		let orig_c = self.cursor_c;
		let orig_a = self.attr;
		self.cursor_r = 24;
		self.cursor_c = 0;
		self.setattr(attr);
		self.print(s);

		self.setattr(orig_a);
		self.cursor_r = orig_r;
		self.cursor_c = orig_c;
	}

	pub fn setattr(&mut self, attr: u8) {
		self.attr = attr;
	}
}

impl fmt::Write for CGAScreen {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.print(s);
		Ok(())
	}
}
