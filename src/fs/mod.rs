//! simple in memory fs that is statically linked into the kernel image

// symbols provided by linker
pub mod ustar;
use core::slice;

extern "C" {
	fn ___RAMFS_START__();
	fn ___RAMFS_END__();
}

pub fn test_print_fs_raw() {
	let len = ___RAMFS_END__ as usize - ___RAMFS_START__ as usize;
	let ramfs: &'static [u8] =
		unsafe { slice::from_raw_parts_mut(___RAMFS_START__ as *mut u8, len) };
	ustar::test_ls(ramfs);
}
