//! simple in memory fs that is statically linked into the kernel image

// symbols provided by linker
pub mod ustar;
use crate::proc::loader;
use alloc::vec::Vec;
use core::slice;
use core::str;
use ustar::UstarArchiveIter;
extern "C" {
	fn ___RAMFS_START__();
	fn ___RAMFS_END__();
}

pub use ustar::iter;
pub use ustar::UstarFile as File;

/// get an raw archive of the fstar fs slice
pub fn get_archive<'a>() -> &'a [u8] {
	let len = ___RAMFS_END__ as usize - ___RAMFS_START__ as usize;
	let ramfs: &[u8] = unsafe { slice::from_raw_parts_mut(___RAMFS_START__ as *mut u8, len) };
	ramfs
}

pub fn test_print_fs_raw() {
	let len = ___RAMFS_END__ as usize - ___RAMFS_START__ as usize;
	let ramfs: &'static [u8] =
		unsafe { slice::from_raw_parts_mut(___RAMFS_START__ as *mut u8, len) };
	ustar::test_ls(ramfs);
}

pub fn cat(f: &File) {
	match str::from_utf8(f.file) {
		Ok(s) => {
			println!("{}", s)
		}
		_ => {
			// print raw
			// println!("{} is not a text file", f.hdr.name());
			loader::cat_elf(f);
		}
	}
}

pub fn test_ls(archive: &[u8]) {
	for f in ustar::iter(archive) {
		println!(
			"{}:{} - {:6} bytes {}",
			f.hdr.owner(),
			f.hdr.owner_group(),
			f.hdr.size(),
			f.hdr.name()
		);
	}
}
