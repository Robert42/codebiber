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

pub fn crc32(buf: &[u8], crc: u32) -> u32
{
	let mut crc = !crc;

	for byte in buf.iter().copied() {
		crc = CRC32_TABLE[(byte as u32 ^ (crc as u32 & 0xFF)) as usize] ^ (crc >> 8);
	}

	return !crc;
}
