//! a very simple virtual memory manager

use alloc::string::String;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::fmt;
use core::ops::Range;

pub struct VMMan {
	pub vmas: Vec<VMArea>,
}

impl VMMan {
	pub fn new() -> Self {
		Self { vmas: Vec::<VMArea>::new() }
	}
}

bitflags! {
	pub struct VMPerms: u8 {
		const NONE = 0;
		const R = 1 << 0;
		const W = 1 << 1;
		const X = 1 << 2;
	}
}

impl fmt::Debug for VMPerms {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}{}{}",
			if self.contains(Self::R) { "R" } else { "-" },
			if self.contains(Self::W) { "W" } else { "-" },
			if self.contains(Self::X) { "X" } else { "-" }
		)
	}
}

pub struct VMArea {
	pub vm_range: Range<u64>,
	pub tag: String,
	pub user_perms: VMPerms,
	pub backing: VMType,
}

impl fmt::Debug for VMArea {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{:016X}-{:016X} {:?} - {:?} - {}",
			self.vm_range.start, self.vm_range.end, self.user_perms, self.backing, self.tag
		)
	}
}

pub enum VMType {
	ANOM,
	FILE(&'static [u8]),
	// NONE for device memory mappings
	NONE,
}

impl fmt::Debug for VMType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::ANOM => "ANOM",
				Self::FILE(_) => "FILE",
				Self::NONE => "DEV",
			},
		)
	}
}
