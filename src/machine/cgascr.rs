use crate::arch::x86_64::misc::*;
use crate::machine::device_io::*;
use core::{fmt, ptr, slice, str};

// TODO this is a "hard copy" of the c code, making little use
// of the rust features. May rework this into cleaner code...
//
// I would consider these cga parameters constant.
// the scroll() and clear() works with the assumption
// that the CGAScreen memory buffer is 64-bit aligned
// luckily it is.
// if any changes, you'd better hope the assumption still
// holds
// For each character, it takes 2 byte in the buffer
// (one for char and one for attribute)
// Therefore the MAX_COLS should be a multiple of 4
const MAX_COLS: usize = 80;
const MAX_ROWS: usize = 25;
const CGA_BUFFER_START: *mut u8 = 0xb8000 as *mut u8;
const CGA_BUFFER_BYTE_SIZE: usize = MAX_COLS * MAX_ROWS * 2;

// THESE TWO ARE USED TO DO BATCH OPERATIONS ON CGA BUFFER
// MEMORY, HOPEFULLY MAKE IT FASTER.
// I.E. SETTING 4 CHARACTERS AT ONCE.
const CGA_BUFFER_START_64: *mut u64 = 0xb8000 as *mut u64;
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

#[allow(dead_code)]
impl CGAScreen {
	pub fn new() -> Self {
		Self {
			cga_mem: unsafe {
				slice::from_raw_parts_mut(CGA_BUFFER_START, 2 * MAX_COLS * MAX_ROWS)
			},
			cursor_r: 0,
			cursor_c: 0,
			attr: 0x0f,
			iport: IOPort::new(IR_PORT),
			dport: IOPort::new(DR_PORT),
		}
	}

	#[inline(always)]
	fn cal_offset(&self, row: usize, col: usize) -> usize {
		col + row * MAX_COLS
	}

	#[inline(always)]
	pub fn show(&self, row: usize, col: usize, c: char, attr: u8) {
		let index = self.cal_offset(row, col);

		unsafe {
			*CGA_BUFFER_START.offset(index as isize * 2) = c as u8;
			*CGA_BUFFER_START.offset(index as isize * 2 + 1) = attr;
		}
	}

	pub fn putchar(&mut self, ch: char) {
		// TODO use match syntax.
		if ch == '\n' {
			self.cursor_r += 1;
			self.cursor_c = 0;
			self._check_scroll();
		} else {
			self.show(self.cursor_r, self.cursor_c, ch, self.attr);
			self.cursor_c += 1;
			if self.cursor_c >= MAX_COLS {
				self.cursor_c = 0;
				self.cursor_r += 1;
				self._check_scroll();
			}
		}

		self.setpos(self.cursor_r, self.cursor_c);
	}

	#[inline(always)]
	fn _check_scroll(&mut self) {
		if self.cursor_r >= MAX_ROWS {
			self.scroll(1);
			self.cursor_r -= 1;
		}
	}

	pub fn scroll(&self, lines: u32) {
		// TODO
		// sanity check
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

	pub fn setpos(&mut self, row: usize, col: usize) {
		// io ports for instruction register and data register
		let offset = self.cal_offset(row, col);
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

	pub fn show_coners(&self) {
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
