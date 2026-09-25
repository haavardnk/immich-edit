use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::ifd::ExifTagGroup;
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

pub fn without_location(meta: &Metadata) -> Metadata {
    let mut out = Metadata::new();
    meta.into_iter()
        .filter(|t| t.get_group() != ExifTagGroup::GPS)
        .for_each(|t| out.set_tag(t.clone()));
    out
}

fn detect(data: &[u8]) -> Option<FileExtension> {
    let mut cursor = std::io::Cursor::new(data);
    FileExtension::auto_detect(&mut cursor)
}

const MAX_TAG_BYTES: usize = 4096;
const APP1_TIFF_BUDGET: usize = u16::MAX as usize - 2 - 6;
const SKIP_TAGS: [u16; 15] = [
    0x00fe, 0x00ff, 0x0100, 0x0101, 0x0102, 0x0103, 0x0106, 0x0111, 0x0115, 0x0116, 0x0117, 0x011c,
    0x0201, 0x0202, 0x0212,
];
const BULKY_TAGS: [u16; 2] = [0x927c, 0x9286];
const CAPTURE_TAGS: [u16; 24] = [
    0x010f, 0x0110, 0x013b, 0x8298, 0x829a, 0x829d, 0x8822, 0x8827, 0x8833, 0x9003, 0x9004, 0x9010,
    0x9011, 0x9012, 0x9204, 0x9207, 0x9209, 0x920a, 0x9291, 0xa402, 0xa403, 0xa405, 0xa433, 0xa434,
];

fn is_copyable(tag: &ExifTag) -> bool {
    tag.is_writable()
        && !matches!(tag, ExifTag::Orientation(_))
        && !SKIP_TAGS.contains(&tag.as_u16())
        && tag.value_as_u8_vec(&Endian::Little).len() <= MAX_TAG_BYTES
}

fn is_capture_or_gps(tag: &ExifTag) -> bool {
    match tag.get_group() {
        ExifTagGroup::GPS => true,
        ExifTagGroup::GENERIC | ExifTagGroup::EXIF => CAPTURE_TAGS.contains(&tag.as_u16()),
        _ => false,
    }
}

fn write_within_budget(
    bytes: &mut Vec<u8>,
    tags: &[ExifTag],
    file_extension: FileExtension,
) -> crate::PipelineResult<()> {
    let stages: [fn(&ExifTag) -> bool; 3] = [
        |_| true,
        |t| !BULKY_TAGS.contains(&t.as_u16()),
        is_capture_or_gps,
    ];
    let capped = matches!(file_extension, FileExtension::JPEG);
    for keep in stages {
        let mut m = Metadata::new();
        tags.iter()
            .filter(|t| keep(t))
            .for_each(|t| m.set_tag(t.clone()));
        m.set_tag(ExifTag::Orientation(vec![1]));
        let fits = !capped
            || m.encode()
                .map_err(|e| PipelineError::Encode(format!("exif: {e}")))?
                .len()
                <= APP1_TIFF_BUDGET;
        if fits {
            return m
                .write_to_vec(bytes, file_extension)
                .map_err(|e| PipelineError::Encode(format!("exif: {e}")));
        }
    }
    Err(PipelineError::Encode(
        "exif: capture tags alone exceed the 64 KB APP1 limit".into(),
    ))
}

pub fn inject(
    bytes: &mut Vec<u8>,
    exif: &Metadata,
    file_extension: FileExtension,
) -> crate::PipelineResult<()> {
    let tags: Vec<ExifTag> = exif
        .into_iter()
        .filter(|t| is_copyable(t))
        .cloned()
        .collect();
    let original = bytes.clone();
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_within_budget(bytes, &tags, file_extension)
    }));
    match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            *bytes = original;
            Err(e)
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
    use crate::encode::{ImageRgb8, encode_jpeg_rgb, encode_png8};
    use crate::frame::{JpegSubsampling, OutputColorSpace, PngCompression};

    fn tiny_jpeg() -> Vec<u8> {
        let rgb = vec![128u8; 16 * 16 * 3];
        let img = ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        };
        encode_jpeg_rgb(img, 90, JpegSubsampling::Chroma420, OutputColorSpace::SRgb).unwrap()
    }

    fn text() -> String {
        "x".repeat(4090)
    }

    fn blob() -> Vec<u8> {
        vec![0x5a; 4090]
    }

    fn app1_len(jpeg: &[u8]) -> usize {
        if jpeg[2..4] != [0xff, 0xe1] {
            panic!("APP1 does not follow SOI");
        }
        u16::from_be_bytes([jpeg[4], jpeg[5]]) as usize
    }

    fn oversized(extra: &[ExifTag]) -> Metadata {
        let capture = [
            ExifTag::Make(text()),
            ExifTag::Model(text()),
            ExifTag::LensMake(text()),
            ExifTag::LensModel(text()),
            ExifTag::Artist(text()),
            ExifTag::Copyright(text()),
            ExifTag::GPSAreaInformation(blob()),
            ExifTag::GPSLatitude(vec![59u32.into(); 3]),
            ExifTag::MakerNote(blob()),
            ExifTag::UserComment(blob()),
        ];
        let other = [
            ExifTag::ImageDescription(text()),
            ExifTag::Software(text()),
            ExifTag::SpectralSensitivity(text()),
            ExifTag::RelatedSoundFile(text()),
            ExifTag::ImageUniqueID(text()),
            ExifTag::OwnerName(text()),
            ExifTag::SerialNumber(text()),
            ExifTag::LensSerialNumber(text()),
        ];
        let mut src = Metadata::new();
        capture
            .iter()
            .chain(&other)
            .chain(extra)
            .for_each(|t| src.set_tag(t.clone()));
        if src.encode().unwrap().len() <= APP1_TIFF_BUDGET {
            panic!("fixture fits APP1 without shrinking");
        }
        src
    }

    #[test]
    fn inject_shrinks_exif_to_fit_app1() {
        let more = [
            ExifTag::CFAPattern(blob()),
            ExifTag::CompositeImageExposureTimes(blob()),
        ];
        let cases: [(&[ExifTag], &[u16], &[u16]); 2] = [
            (&[], &[0x010e, 0x0131, 0x010f, 0x0002], &[0x927c, 0x9286]),
            (
                &more,
                &[0x010f, 0x013b, 0x8298, 0x001c, 0x0002],
                &[0x010e, 0x0131, 0xa302, 0x927c],
            ),
        ];
        for (extra, kept, dropped) in cases {
            let src = oversized(extra);
            let mut bytes = tiny_jpeg();
            inject(&mut bytes, &src, FileExtension::JPEG).unwrap();

            if bytes[4 + app1_len(&bytes)] != 0xff {
                panic!("APP1 length does not land on the next marker");
            }
            let parsed = parse(&bytes).expect("output has parseable exif");
            let ids: Vec<u16> = parsed.into_iter().map(|t| t.as_u16()).collect();
            let missing: Vec<&u16> = kept.iter().filter(|id| !ids.contains(id)).collect();
            let leaked: Vec<&u16> = dropped.iter().filter(|id| ids.contains(id)).collect();
            if !missing.is_empty() || !leaked.is_empty() {
                panic!("missing {missing:04x?}, leaked {leaked:04x?}");
            }
        }
    }

    #[test]
    fn inject_keeps_every_tag_outside_jpeg() {
        let rgb = vec![128u8; 16 * 16 * 3];
        let img = ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        };
        let mut bytes = encode_png8(img, PngCompression::Default, OutputColorSpace::SRgb).unwrap();
        let ext = FileExtension::PNG {
            as_zTXt_chunk: true,
        };
        inject(&mut bytes, &oversized(&[]), ext).unwrap();

        let parsed = parse(&bytes).expect("output has parseable exif");
        let ids: Vec<u16> = parsed.into_iter().map(|t| t.as_u16()).collect();
        let missing: Vec<&u16> = [0x927c, 0x9286, 0x010e]
            .iter()
            .filter(|id| !ids.contains(id))
            .collect();
        if !missing.is_empty() {
            panic!("png lost {missing:04x?}");
        }
    }

    #[test]
    fn without_location_keeps_the_camera_and_drops_gps() {
        let mut src = Metadata::new();
        src.set_tag(ExifTag::Make("SONY".to_string()));
        src.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
        src.set_tag(ExifTag::GPSLatitudeRef("N".to_string()));

        let stripped = without_location(&src);
        let groups: Vec<ExifTagGroup> = stripped.into_iter().map(|t| t.get_group()).collect();
        let has_make = stripped
            .into_iter()
            .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"));
        if !has_make || groups.contains(&ExifTagGroup::GPS) {
            panic!("expected Make without GPS, got groups {groups:?}");
        }
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
