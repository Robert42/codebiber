///////////////////////////////////////////////////////////////////////////////
//
//  Original Implementation:
//  - Author:     Lasse Collin## crc32 source
//  - Original License: 0BSD
//  - Downloaded from
//    - https://web.archive.org/web/20250704104106/https://tukaani.org/xz/#_licensing
//    - https://web.archive.org/web/20250704103854/https://github.com/tukaani-project/xz
//    - https://web.archive.org/web/20250704104455/https://github.com/tukaani-project/xz/blob/master/COPYING
//    - https://web.archive.org/web/20250704104738/https://github.com/tukaani-project/xz/blob/master/COPYING.0BSD
//    - https://web.archive.org/web/20250704103532/https://github.com/tukaani-project/xz/blob/master/src/liblzma/check/crc32_small.c
//    - https://web.archive.org/web/20250704103711/https://raw.githubusercontent.com/tukaani-project/xz/refs/heads/master/src/liblzma/check/crc32_small.c
//
//  Translated to Rust on 2025-07-04 by Robert Hildebrandt
//
///////////////////////////////////////////////////////////////////////////////

pub const KEY_LEN : usize = 4;
pub const CHECKSUM_TEXT_LEN : usize = KEY_LEN * 2;
pub fn hash(buf: &[u8]) -> Hash
{
	return crc32(buf, 0);
}

pub fn fmt(h: Hash) -> String
{
	return format!("{:08x}", h);
}

pub fn fmt_write(out: &mut String, h: Hash)
{
	use std::fmt::Write;
	write!(out, "{:08x}", h).unwrap();
}

pub type Hash = u32;

const CRC32_TABLE : [u32 ; 256] = crc32_init();

const fn crc32_init() -> [u32 ; 256]
{
	const POLY32 : u32 = 0xEDB88320;
	let mut crc32_table = [0_u32 ; 256];

	let mut b = 0;
	while b < 256 {
		let mut r : u32 = b as u32;
		let mut i = 0;
		while i < 8 {
			if (r & 1) != 0
			{
				r = (r >> 1) ^ POLY32;
			}else
			{
				r >>= 1;
			}
			i += 1;
		}

		crc32_table[b] = r;
		b += 1;
	}

	return crc32_table;
}

fn crc32(buf: &[u8], crc: u32) -> u32
{
	let mut crc = !crc;

	for byte in buf.iter().copied() {
		crc = CRC32_TABLE[(byte as u32 ^ (crc as u32 & 0xFF)) as usize] ^ (crc >> 8);
	}

	return !crc;
}

#[test]
fn test()
{
	let samples : Vec<&[u8]> = vec![b"137\n", ];
	for bytes in samples.into_iter()
	{
		println!("assert_eq!(crc32({bytes:?}, 0), 0x{:08x});", crc32(bytes, 0));
	}

  assert_eq!(hash(&[120]), 0x8cdc1683);
  assert_eq!(hash(&[120, 121]), 0x8fe62899);
  assert_eq!(hash(&[120, 121, 122]), 0xeb8eba67);
  assert_eq!(hash(b"42\n  137\n1337\n"), 0xf1656245);
  assert_eq!(hash(b"42\n137\n1337\n"), 0xdda9452f);
  assert_eq!(hash(b"xyz\n"), 0xe1ea7cd2);
  assert_eq!(hash(b"uvw\n"), 0xad729d7f);
  assert_eq!(hash(b"xyuz\nuv\n"), 0x2fddd919);
  assert_eq!(hash(b"xyuz\n<>\n  []\nuv\n"), 0xc1e2b1d0);
  assert_eq!(hash(b"42"), 0x3224B088);
  assert_eq!(hash(b"42\n"), 0xd1862931);
  assert_eq!(hash(b"137\n"), 0x3f2c523b);
  assert_eq!(hash(b"\n"), 0x32d70693);
}
