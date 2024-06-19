//! this asume a read-only; in memory filesystem, and we assume the FS has a
//! static lifetime . This makes slice type much easier.
use alloc::vec::Vec;
use core::str;

// yes, I want this naming, shut up rust.
#[allow(non_camel_case_types)]
pub enum FileType {
	NORMAL,
	HLINK,
	SYMLINK,
	CHAR_DEV,
	BLOCK_DEV,
	DIR,
	FIFO,
	CONT,
	EXTHDR_META,
	EXTHDR_METANXT,
	// RESERVED includes vender specifics ('A' - 'Z')
	RESERVED(u8),
}

impl FileType {
	pub fn from_u8(flag: u8) -> Self {
		match flag as char {
			'0' | '\0' => Self::NORMAL,
			'1' => Self::HLINK,
			'2' => Self::SYMLINK,
			'3' => Self::CHAR_DEV,
			'5' => Self::DIR,
			'6' => Self::FIFO,
			'7' => Self::CONT,
			'g' => Self::EXTHDR_META,
			'x' => Self::EXTHDR_METANXT,
			_ => Self::RESERVED(flag),
		}
	}
}

pub struct UstarFile {
	pub hdr: FileHdr,
	pub file: &'static [u8],
}

pub fn test_ls(archive: &'static [u8]) {
	let files = ls(archive);
	for f in files {
		println!(
			"{}:{} - {:6} bytes {}",
			f.hdr.owner(),
			f.hdr.owner_group(),
			f.hdr.size(),
			f.hdr.name()
		);
	}
}

pub fn ls(archive: &'static [u8]) -> Vec<UstarFile> {
	let mut ptr: usize = 0;
	let mut files = Vec::<UstarFile>::new();
	loop {
		if ptr >= archive.len() {
			break;
		}
		if let Some(hdr) = FileHdr::from_slice(&archive[ptr..]) {
			if !hdr.is_ustar() {
				break;
			}
			// "valid" ustar file
			let file_sz = hdr.size() as usize;
			files.push(UstarFile {
				hdr,
				file: &archive[(ptr + 512)..(ptr + 512 + file_sz)],
			});
			// get the next file hdr: jump over current hdr and file, round up
			// to 512 bytes
			ptr += ((((file_sz + 511) / 512) + 1) * 512) as usize;
		} else {
			break;
		}
	}
	files
}

pub struct FileHdr(&'static [u8]);
impl FileHdr {
	pub fn name(&self) -> &str {
		let file_name = to_cstr(&self.0[0..100]);
		str::from_utf8(file_name.unwrap())
			.unwrap()
			.trim_matches(char::from(0))
	}
	pub fn owner(&self) -> &str {
		let user_name = to_cstr(&self.0[265..265 + 32]);
		str::from_utf8(user_name.unwrap()).unwrap()
	}
	pub fn owner_group(&self) -> &str {
		let user_name = to_cstr(&self.0[297..297 + 32]);
		str::from_utf8(user_name.unwrap()).unwrap()
	}
	pub fn size(&self) -> u32 {
		let sz = &self.0[124..124 + 11];
		let n_sz = oct2bin(sz);
		n_sz
	}
	pub fn is_ustar(&self) -> bool {
		if let Ok(magic) = str::from_utf8(&self.0[257..(257 + 5)]) {
			return magic == "ustar";
		}
		return false;
	}
	pub fn from_slice(ustar_slice: &'static [u8]) -> Option<Self> {
		if ustar_slice.len() < 512 {
			return None;
		}
		Some(Self(&ustar_slice[0..512]))
	}
}

fn oct2bin(s: &[u8]) -> u32 {
	let mut n: u32 = 0;
	for u in s {
		n *= 8;
		let d = *u - ('0' as u8);
		n += d as u32;
	}
	n
}

// this is kinda ugly, is there a builtin funciton?
fn to_cstr(s: &[u8]) -> Option<&[u8]> {
	let mut end = 0;
	for c in s {
		if *c == 0 {
			if end == 0 {
				return None;
			} else {
				break;
			}
		}
		end += 1;
	}
	return Some(&s[0..=end]);
}
