use crate::machine::device_io::*;
use crate::{arch::x86_64::misc::*, P2V};
use core::{fmt, ptr, slice, str};

// I would consider these cga parameters constant.
// the scroll() and clear() works with the assumption
// that the CGAScreen memory buffer is 64-bit aligned
// luckily it is.
// if any changes, you'd better hope the assumption still
// holds
// For each character, it takes 2 byte in the buffer
// (one for char and one for attribute)
// Therefore the MAX_COLS should be a multiple of 4
// broken..
//
// TODO: clean me up
const MAX_COLS: usize = 80;
const MAX_ROWS: usize = 25;
const CGA_BUFFER_START: *mut u8 = P2V(0xb8000).unwrap() as *mut u8;
const CGA_BUFFER_START_64: *mut u64 = P2V(0xb8000).unwrap() as *mut u64;
const CGA_BUFFER_BYTE_SIZE: usize = MAX_COLS * MAX_ROWS * 2;

// THESE TWO ARE USED TO DO BATCH OPERATIONS ON CGA BUFFER
// MEMORY, HOPEFULLY MAKE IT FASTER.
// I.E. SETTING 4 CHARACTERS AT ONCE.
const CGA_BUFFER_QWORD_SIZE: usize = CGA_BUFFER_BYTE_SIZE / 8;
const CGA_BUFFER_QWORDS_PER_ROW: usize = MAX_COLS / 4;

const IR_PORT: u16 = 0x3d4;
const DR_PORT: u16 = 0x3d5;

#[allow(dead_code)]
pub struct CGAScreen {
	pub cga_mem: &'static mut [u8],
	cursor_r: usize,
	cursor_c: usize,
	attr: u8,
	iport: IOPort,
	dport: IOPort,
}

#[inline(always)]
pub fn cal_offset(row: usize, col: usize) -> usize {
	col + row * MAX_COLS
}

#[allow(dead_code)]
impl CGAScreen {
	pub fn new() -> Self {
		let cga = Self {
			cga_mem: unsafe {
				slice::from_raw_parts_mut(CGA_BUFFER_START, 2 * MAX_COLS * MAX_ROWS)
			},
			cursor_r: 0,
			cursor_c: 0,
			attr: 0x0f,
			iport: IOPort::new(IR_PORT),
			dport: IOPort::new(DR_PORT),
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

	/// print a char at the current cursor location and update the
	/// cursor. Scroll the screen if needed.
	pub fn putchar(&mut self, ch: char) {
		// TODO align to next tabstop on \t ... but this will make backspace
		// trickier ..
		match ch {
			'\n' => {
				self.cursor_r += 1;
				self.cursor_c = 0;
				self._check_scroll();
			}
			_ => {
				self.show(self.cursor_r, self.cursor_c, ch, self.attr);
				self.cursor_c += 1;
				if self.cursor_c >= MAX_COLS {
					self.cursor_c = 0;
					self.cursor_r += 1;
					self._check_scroll();
				}
			}
		}
		// update the on-screen cursor.
		self.setpos(self.cursor_r, self.cursor_c);
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
		self.setpos(self.cursor_r, self.cursor_c);
		self.show(self.cursor_r, self.cursor_c, 0 as char, self.attr);
	}

	#[inline(always)]
	fn _check_scroll(&mut self) {
		if self.cursor_r >= MAX_ROWS {
			self.scroll(1);
			self.cursor_r -= 1;
		}
	}

	pub fn scroll(&self, lines: u32) {
		if lines >= MAX_ROWS as u32 {
			self.clear();
		}

		if lines == 0 {
			return;
		}
		// behold the magic ... (oh fuck me)
		let mut i: usize = lines as usize - 1;
		loop {
			if i == MAX_ROWS {
				break;
			}
			let offset_src = (i * CGA_BUFFER_QWORDS_PER_ROW) as isize;
			let offset_dist = offset_src - (lines * CGA_BUFFER_QWORDS_PER_ROW as u32) as isize;
			unsafe {
				ptr::copy_nonoverlapping(
					CGA_BUFFER_START_64.offset(offset_src),
					CGA_BUFFER_START_64.offset(offset_dist),
					CGA_BUFFER_QWORDS_PER_ROW,
				);
			}
			i += 1;
		}

		i = MAX_ROWS - lines as usize;
		loop {
			if i == MAX_ROWS {
				break;
			}
			self.clearline(i);
			i += 1;
		}
		// clear the remaining lines:
	}

	pub fn clear(&self) {
		// remember to swap the endian..
		let b: u8 = self.attr;
		let mut base: u64 = (b as u64) << 8;
		base += base << 16;
		base += base << 32;

		for i in 0..CGA_BUFFER_QWORD_SIZE {
			unsafe { *CGA_BUFFER_START_64.offset(i as isize) = base }
		}
	}

	pub fn reset(&mut self) {
		self.clear();
		self.cursor_c = 0;
		self.cursor_r = 0;
		self.setpos(0, 0);
	}

	fn clearline(&self, line: usize) {
		let b: u8 = self.attr;
		let mut base: u64 = (b as u64) << 8;
		base += base << 16;
		base += base << 32;
		let start_offset_qw: isize = (line as isize) * CGA_BUFFER_QWORDS_PER_ROW as isize;
		let end_offset_qw: isize = start_offset_qw + CGA_BUFFER_QWORDS_PER_ROW as isize;
		unsafe {
			for i in start_offset_qw..end_offset_qw {
				*CGA_BUFFER_START_64.offset(i) = base;
			}
		}
	}

	/// the on-screen cursor position is decoupled with the system's own book
	/// keeping, i.e. we update the cursor position based on our own record, but
	/// we never read this position back.
	pub fn setpos(&mut self, row: usize, col: usize) {
		// io ports for instruction register and data register
		let offset = cal_offset(row, col);
		// set lower byte
		self.iport.outb(15 as u8);
		delay();
		self.dport.outb(offset as u8);
		// set higher byte
		self.iport.outb(14 as u8);
		self.dport.outb((offset >> 8) as u8);
		self.cursor_r = row;
		self.cursor_c = col;
	}

	// make cursor blink (is this necessary??)
	pub fn init_cursor(&self) {
		self.iport.outb(0x0a);
		delay();
		let mut d = self.dport.inb();
		delay();
		d = d & 0xc0;
		d = d | 0xe;
		self.dport.outb(d);
		delay();
		self.iport.outb(0x0b);
		delay();
		let mut d = self.dport.inb();
		d = d & 0xe0;
		d = d | 0xf;
		self.dport.outb(d);
	}

	pub fn getpos_xy(&self, row: &mut u32, col: &mut u32) {
		let offset = self.getpos_offset();
		*row = offset % MAX_COLS as u32;
		*col = offset / MAX_COLS as u32;
	}

	#[allow(arithmetic_overflow)]
	pub fn getpos_offset(&self) -> u32 {
		// read higher byte
		self.iport.outb(14 as u8);
		let mut offset = self.dport.inb();
		offset = offset << 8;
		// read lower byte
		self.iport.outb(15 as u8);
		offset += self.dport.inb();
		offset as u32
	}

	pub fn show_corners(&mut self) {
		// TODO replace hardcoded
		self.show(0, 0, 0xda as char, self.attr);
		self.show(0, 79, 0xbf as char, self.attr);
		self.show(24, 0, 0xc0 as char, self.attr);
		self.show(24, 79, 0xd9 as char, self.attr);
	}

	pub fn print(&mut self, s: &str) {
		for c in s.bytes() {
			self.putchar(c as char);
		}
	}

	/// this is helpful for some helper text.
	/// this will clear the bottom line
	/// this will not update the cursor location.
	pub fn print_at_bottom(&mut self, s: &str, attr: u8) {
		self.clearline(24);
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
