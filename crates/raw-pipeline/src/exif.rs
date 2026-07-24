use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

use crate::PipelineError;
use crate::frame::OrientFlips;

pub fn orientation(meta: &Metadata) -> Option<OrientFlips> {
    let tag = meta.get_tag(&ExifTag::Orientation(vec![])).next()?;
    if let ExifTag::Orientation(vals) = tag {
        let v = *vals.first()?;
        Some(match v {
            2 => (false, true, false),
            3 => (false, true, true),
            4 => (false, false, true),
            5 => (true, false, false),
            6 => (true, false, true),
            7 => (true, true, true),
            8 => (true, true, false),
            _ => (false, false, false),
        })
    } else {
        None
    }
}

pub fn parse(data: &[u8]) -> Option<Metadata> {
    let ext = detect(data)?;
    let vec = data.to_vec();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Metadata::new_from_vec(&vec, ext).ok()
    }))
    .ok()
    .flatten()
}

fn detect(data: &[u8]) -> Option<FileExtension> {
    let mut cursor = std::io::Cursor::new(data);
    FileExtension::auto_detect(&mut cursor)
}

pub fn inject(
    bytes: &mut Vec<u8>,
    exif: &Metadata,
    file_extension: FileExtension,
) -> crate::PipelineResult<()> {
    const MAX_TAG_BYTES: usize = 4096;
    const SKIP_TAGS: [u16; 15] = [
        0x00fe, 0x00ff, 0x0100, 0x0101, 0x0102, 0x0103, 0x0106, 0x0111, 0x0115, 0x0116, 0x0117,
        0x011c, 0x0201, 0x0202, 0x0212,
    ];
    let mut m = Metadata::new();
    for tag in exif.into_iter() {
        if !tag.is_writable() || matches!(tag, ExifTag::Orientation(_)) {
            continue;
        }
        if SKIP_TAGS.contains(&tag.as_u16()) {
            continue;
        }
        if tag.value_as_u8_vec(&Endian::Little).len() > MAX_TAG_BYTES {
            continue;
        }
        m.set_tag(tag.clone());
    }
    m.set_tag(ExifTag::Orientation(vec![1]));
    let original = bytes.clone();
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.write_to_vec(bytes, file_extension)
    }));
    match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            *bytes = original;
            Err(PipelineError::Encode(format!("exif: {e}")))
        }
        Err(_) => {
            *bytes = original;
            Err(PipelineError::Encode("exif write panic".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{ImageRgb8, encode_jpeg_rgb};
    use crate::frame::{JpegSubsampling, OutputColorSpace};

    fn tiny_jpeg() -> Vec<u8> {
        let rgb = vec![128u8; 16 * 16 * 3];
        let img = ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        };
        encode_jpeg_rgb(img, 90, JpegSubsampling::Chroma420, OutputColorSpace::SRgb).unwrap()
    }

    #[test]
    fn inject_drops_embedded_preview_strips() {
        let preview = vec![0xABu8; 4_000_000];
        let mut src = Metadata::new();
        src.set_tag(ExifTag::Make("SONY".to_string()));
        src.set_tag(ExifTag::StripOffsets(vec![512], vec![preview]));
        src.set_tag(ExifTag::StripByteCounts(vec![4_000_000]));

        let mut bytes = tiny_jpeg();
        let base_len = bytes.len();
        inject(&mut bytes, &src, FileExtension::JPEG).unwrap();

        if bytes.len() > base_len + 64_000 {
            panic!("embedded preview leaked into exif: {} bytes", bytes.len());
        }

        let parsed = parse(&bytes).expect("output has parseable exif");
        let has_make = parsed
            .into_iter()
            .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"));
        if !has_make {
            panic!("make tag was not preserved");
        }
        let has_strips = parsed
            .into_iter()
            .any(|t| matches!(t, ExifTag::StripOffsets(_, _) | ExifTag::StripByteCounts(_)));
        if has_strips {
            panic!("strip tags were not removed");
        }
    }
}
