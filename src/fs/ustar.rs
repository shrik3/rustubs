//! this asume a read-only; in memory filesystem, and we assume the FS has a
//! static lifetime . This makes slice type much easier.
use core::iter::Iterator;
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

#[derive(Clone)]
pub struct UstarArchiveIter<'a> {
	pub archive: &'a [u8],
	pub iter_curr: usize,
}

/// gives you an ustar file iterator over an read only u8 slice archive
pub fn iter(archive: &[u8]) -> UstarArchiveIter<'_> {
	UstarArchiveIter { archive, iter_curr: 0 }
}

impl<'a> Iterator for UstarArchiveIter<'a> {
	type Item = UstarFile<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		if self.iter_curr >= self.archive.len() {
			return None;
		}

		let hdr = FileHdr::from_slice(&self.archive[self.iter_curr..]);
		let hdr = hdr?;
		if !hdr.is_ustar() {
			return None;
		}
		let file_sz = hdr.size() as usize;
		let ret = Some(UstarFile {
			hdr: hdr.clone(),
			file: &self.archive[(self.iter_curr + 512)..(self.iter_curr + 512 + file_sz)],
		});
		self.iter_curr += (((file_sz + 511) / 512) + 1) * 512;
		return ret;
	}
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

// Actually this is chanllenging: how do you justify lifetimes of these things?
// - a "opened file"
// - page cache that buffers the file content
// - the reference to the backing file in the VMA...
// well, it seems we need the abstraction of file descriptors, if we can't
// guarantee the lifetime of a file to outlive ther tasks
#[derive(Clone)]
pub struct UstarFile<'a> {
	pub hdr: FileHdr<'a>,
	pub file: &'a [u8],
}

#[derive(Clone)]
pub struct FileHdr<'a>(&'a [u8]);
impl<'a> FileHdr<'a> {
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
		oct2bin(sz)
	}
	pub fn is_ustar(&self) -> bool {
		if let Ok(magic) = str::from_utf8(&self.0[257..(257 + 5)]) {
			return magic == "ustar";
		}
		return false;
	}
	pub fn from_slice(ustar_slice: &'a [u8]) -> Option<Self> {
		if ustar_slice.len() < 512 {
			return None;
		}
		Some(Self(&ustar_slice[0..512]))
	}
}

/// helper function to convert oct literals into number
fn oct2bin(s: &[u8]) -> u32 {
	let mut n: u32 = 0;
	for u in s {
		n *= 8;
		let d = *u - b'0';
		n += d as u32;
	}
	n
}

/// cut the slice at the first null
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
