use little_exif::metadata::Metadata;

use super::parse_tiff;

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    app1_segments(data).find_map(|segment| parse_tiff(segment.strip_prefix(b"Exif\0\0")?))
}

fn app1_segments(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut pos = 2usize;
    std::iter::from_fn(move || {
        loop {
            if *data.get(pos)? != 0xFF {
                return None;
            }
            let marker = *data.get(pos + 1)?;
            match marker {
                0xFF => {
                    pos += 1;
                    continue;
                }
                0x01 | 0xD0..=0xD8 => {
                    pos += 2;
                    continue;
                }
                0xD9 | 0xDA => return None,
                _ => {}
            }
            let len = u16::from_be_bytes([*data.get(pos + 2)?, *data.get(pos + 3)?]) as usize;
            let payload = data.get(pos + 4..pos + 2 + len)?;
            pos += 2 + len;
            if marker == 0xE1 {
                return Some(payload);
            }
        }
    })
}
