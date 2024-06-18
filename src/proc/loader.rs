//! a simple loader for statically linked elf.
use crate::fs::File;
use xmas_elf::ElfFile;
pub fn cat_elf(f: &File) {
	let elf = ElfFile::new(f.file).unwrap();
	println!("{:?}", elf.header);
}
