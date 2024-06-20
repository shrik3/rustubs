//! simple in memory fs that is statically linked into the kernel image

// symbols provided by linker
pub mod ustar;
use crate::proc::loader;
use core::slice;
use core::str;
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

pub fn cat(f: &File) {
	match str::from_utf8(f.file) {
		Ok(s) => println!("{}", s),
		_ => loader::cat_elf(f),
	}
}
