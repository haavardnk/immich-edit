mod raw_metadata;
mod repair;

use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;

use crate::PipelineError;
use crate::frame::{OrientFlips, OutputColorSpace};

pub use raw_metadata::from_raw_metadata;

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

pub(crate) fn parse_tiff(tiff: &[u8]) -> Option<Metadata> {
    let vec = repair::repair(tiff)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Metadata::new_from_vec(&vec, FileExtension::TIFF).ok()
    }))
    .ok()
    .flatten()
    .filter(|meta| meta.into_iter().next().is_some())
}

pub fn merge(primary: Option<Metadata>, fallback: Option<Metadata>) -> Option<Metadata> {
    let Some(mut primary) = primary else {
        return fallback;
    };
    let Some(fallback) = fallback else {
        return Some(primary);
    };
    let present: Vec<(u16, ExifTagGroup)> = primary
        .into_iter()
        .map(|t| (t.as_u16(), t.get_group()))
        .collect();
    fallback
        .into_iter()
        .filter(|t| !present.contains(&(t.as_u16(), t.get_group())))
        .for_each(|t| primary.set_tag(t.clone()));
    Some(primary)
}

const MAX_TAG_BYTES: usize = 4096;
const LAYOUT_TAGS: [std::ops::RangeInclusive<u16>; 15] = [
    0x00fe..=0x0103,
    0x0106..=0x010a,
    0x0111..=0x0111,
    0x0115..=0x0119,
    0x011c..=0x011c,
    0x0122..=0x0125,
    0x012d..=0x012d,
    0x013d..=0x0146,
    0x014c..=0x014c,
    0x0150..=0x0156,
    0x015b..=0x015b,
    0x0200..=0x0209,
    0x0211..=0x0214,
    0x8773..=0x8773,
    0x935c..=0x935c,
];
const STRIPPED_TAGS: [u16; 6] = [0x014a, 0x02bc, 0x83bb, 0x8649, 0x927c, 0xa420];
const DNG_PRIVATE: std::ops::RangeInclusive<u16> = 0xc612..=0xc7ff;
const BULKY_TAGS: [u16; 1] = [0x9286];
const CAPTURE_TAGS: [u16; 24] = [
    0x010f, 0x0110, 0x013b, 0x8298, 0x829a, 0x829d, 0x8822, 0x8827, 0x8833, 0x9003, 0x9004, 0x9010,
    0x9011, 0x9012, 0x9204, 0x9207, 0x9209, 0x920a, 0x9291, 0xa402, 0xa403, 0xa405, 0xa433, 0xa434,
];

#[derive(Debug, Clone)]
pub struct ExifBlock {
    pub tags: Metadata,
    pub tiff: Vec<u8>,
}

fn is_copyable(tag: &ExifTag) -> bool {
    let id = tag.as_u16();
    tag.is_writable()
        && tag.get_group() != ExifTagGroup::INTEROP
        && !matches!(tag, ExifTag::Orientation(_))
        && !LAYOUT_TAGS.iter().any(|range| range.contains(&id))
        && !STRIPPED_TAGS.contains(&id)
        && !DNG_PRIVATE.contains(&id)
        && tag.value_as_u8_vec(&Endian::Little).len() <= MAX_TAG_BYTES
}

fn is_capture_or_gps(tag: &ExifTag) -> bool {
    match tag.get_group() {
        ExifTagGroup::GPS => true,
        ExifTagGroup::GENERIC | ExifTagGroup::EXIF => CAPTURE_TAGS.contains(&tag.as_u16()),
        _ => false,
    }
}

fn within_budget(
    tags: &[ExifTag],
    rewritten: &[ExifTag],
    budget: Option<usize>,
) -> crate::PipelineResult<ExifBlock> {
    let stages: [fn(&ExifTag) -> bool; 3] = [
        |_| true,
        |t| !BULKY_TAGS.contains(&t.as_u16()),
        is_capture_or_gps,
    ];
    for keep in stages {
        let mut meta = Metadata::new();
        tags.iter()
            .filter(|t| keep(t))
            .chain(rewritten)
            .for_each(|t| meta.set_tag(t.clone()));
        let tiff = meta
            .encode()
            .map_err(|e| PipelineError::Encode(format!("exif: {e}")))?;
        if budget.is_none_or(|limit| tiff.len() <= limit) {
            return Ok(ExifBlock { tags: meta, tiff });
        }
    }
    Err(PipelineError::Encode(
        "exif: capture tags alone exceed the 64 KB APP1 limit".into(),
    ))
}

pub fn block(
    exif: &Metadata,
    location: bool,
    budget: Option<usize>,
    color_space: OutputColorSpace,
    (width, height): (u32, u32),
) -> crate::PipelineResult<ExifBlock> {
    let tags: Vec<ExifTag> = exif
        .get_ifds()
        .iter()
        .filter(|ifd| ifd.get_generic_ifd_nr() == 0)
        .flat_map(|ifd| ifd.get_tags())
        .filter(|t| is_copyable(t) && (location || t.get_group() != ExifTagGroup::GPS))
        .cloned()
        .collect();
    let rewritten = [
        ExifTag::Orientation(vec![1]),
        ExifTag::ColorSpace(vec![match color_space {
            OutputColorSpace::SRgb => 1,
            OutputColorSpace::DisplayP3 => 0xffff,
        }]),
        ExifTag::ExifImageWidth(vec![width]),
        ExifTag::ExifImageHeight(vec![height]),
        ExifTag::Software(format!("immich-edit {}", crate::version())),
    ];
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        within_budget(&tags, &rewritten, budget)
    }))
    .unwrap_or_else(|_| Err(PipelineError::Encode("exif: encoder panicked".into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::jpeg::EXIF_BUDGET;
    use crate::encode::{ImageRgb8, encode_jpeg_rgb};
    use crate::frame::{JpegSubsampling, OutputColorSpace};

    fn parse(bytes: &[u8]) -> Option<Metadata> {
        crate::metadata::read(bytes)
    }

    fn jpeg_with(exif: ExifBlock) -> Vec<u8> {
        let rgb = vec![128u8; 16 * 16 * 3];
        let img = ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        };
        encode_jpeg_rgb(
            img,
            90,
            JpegSubsampling::Chroma420,
            OutputColorSpace::SRgb,
            Some(&exif),
        )
        .unwrap()
    }

    fn ids(meta: &Metadata) -> Vec<u16> {
        meta.into_iter().map(|t| t.as_u16()).collect()
    }

    fn sanitized(src: &Metadata, location: bool, budget: Option<usize>) -> ExifBlock {
        block(src, location, budget, OutputColorSpace::SRgb, (16, 16)).unwrap()
    }

    fn text() -> String {
        "x".repeat(4090)
    }

    fn blob() -> Vec<u8> {
        vec![0x5a; 4090]
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
            ExifTag::SubSecTime(text()),
            ExifTag::SubSecTimeDigitized(text()),
            ExifTag::OECF(blob()),
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
        let size = sanitized(&src, true, None).tiff.len();
        if size <= EXIF_BUDGET {
            panic!("fixture fits APP1 without shrinking: {size} bytes");
        }
        src
    }

    #[test]
    fn block_shrinks_exif_to_fit_jpeg_app1() {
        let more = [
            ExifTag::CFAPattern(blob()),
            ExifTag::CompositeImageExposureTimes(blob()),
        ];
        let cases: [(&[ExifTag], &[u16], &[u16]); 2] = [
            (&[], &[0x010e, 0x0131, 0x010f, 0x0002], &[0x9286]),
            (
                &more,
                &[0x010f, 0x0131, 0x013b, 0x8298, 0x001c, 0x0002],
                &[0x010e, 0xa302],
            ),
        ];
        for (extra, kept, dropped) in cases {
            let exif = sanitized(&oversized(extra), true, Some(EXIF_BUDGET));
            let bytes = jpeg_with(exif);
            if bytes[2..4] != [0xff, 0xe1] {
                panic!("APP1 does not follow SOI");
            }
            let app1 = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
            if bytes[4 + app1] != 0xff {
                panic!("APP1 length does not land on the next marker");
            }
            let found = ids(&parse(&bytes).expect("output has parseable exif"));
            let missing: Vec<&u16> = kept.iter().filter(|id| !found.contains(id)).collect();
            let leaked: Vec<&u16> = dropped.iter().filter(|id| found.contains(id)).collect();
            if !missing.is_empty() || !leaked.is_empty() {
                panic!("missing {missing:04x?}, leaked {leaked:04x?}");
            }
        }
    }

    #[test]
    fn block_without_budget_keeps_every_tag() {
        let exif = sanitized(&oversized(&[]), true, None);
        let found = ids(&parse_tiff(&exif.tiff).expect("block parses"));
        let missing: Vec<&u16> = [0x9286, 0x9290, 0x010e]
            .iter()
            .filter(|id| !found.contains(id))
            .collect();
        if !missing.is_empty() {
            panic!("unbudgeted block lost {missing:04x?}");
        }
    }

    fn ifd_entry(tag: u16, format: u16, count: u32, value: [u8; 4]) -> Vec<u8> {
        [
            tag.to_le_bytes().as_slice(),
            &format.to_le_bytes(),
            &count.to_le_bytes(),
            &value,
        ]
        .concat()
    }

    #[test]
    fn parse_tiff_keeps_the_ifd_around_one_malformed_tag() {
        let tiff = [
            b"II\x2a\x00\x08\x00\x00\x00".as_slice(),
            &2u16.to_le_bytes(),
            &ifd_entry(0x010f, 2, 2, *b"X\0\0\0"),
            &ifd_entry(0x8769, 4, 1, 38u32.to_le_bytes()),
            &[0; 4],
            &2u16.to_le_bytes(),
            &ifd_entry(0x829a, 5, 1, 0xffffu32.to_le_bytes()),
            &ifd_entry(0xa301, 1, 1, [1, 0, 0, 0]),
            &[0; 4],
        ]
        .concat();
        let Some(meta) = parse_tiff(&tiff) else {
            panic!("one bad entry discarded the whole EXIF block");
        };
        let found = ids(&meta);
        if !found.contains(&0x010f) || !found.contains(&0xa301) || found.contains(&0x829a) {
            panic!("expected Make and SceneType without ExposureTime, got {found:04x?}");
        }
    }

    #[test]
    fn block_without_location_keeps_the_camera_and_drops_gps() {
        let mut src = Metadata::new();
        src.set_tag(ExifTag::Make("SONY".to_string()));
        src.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
        src.set_tag(ExifTag::GPSLatitudeRef("N".to_string()));

        let exif = sanitized(&src, false, None);
        let parsed = parse_tiff(&exif.tiff).expect("block parses");
        let groups: Vec<ExifTagGroup> = parsed.into_iter().map(|t| t.get_group()).collect();
        let has_make = parsed
            .into_iter()
            .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"));
        if !has_make || groups.contains(&ExifTagGroup::GPS) {
            panic!("expected Make without GPS, got groups {groups:?}");
        }
    }

    #[test]
    fn block_keeps_ifd0_values_over_the_thumbnail_ifd() {
        let mut src = Metadata::new();
        src.set_tag(ExifTag::XResolution(vec![300u32.into()]));
        src.get_ifd_mut(ExifTagGroup::GENERIC, 1)
            .set_tag(ExifTag::XResolution(vec![72u32.into()]));

        let exif = sanitized(&src, true, None);
        let parsed = parse_tiff(&exif.tiff).expect("block parses");
        let resolutions: Vec<u32> = (&parsed)
            .into_iter()
            .filter_map(|t| match t {
                ExifTag::XResolution(v) => v.first().map(|r| r.nominator / r.denominator),
                _ => None,
            })
            .collect();
        if resolutions != [300] {
            panic!("expected the IFD0 resolution only, got {resolutions:?}");
        }
    }

    #[test]
    fn block_drops_embedded_preview_strips() {
        let preview = vec![0xABu8; 4_000_000];
        let mut src = Metadata::new();
        src.set_tag(ExifTag::Make("SONY".to_string()));
        src.set_tag(ExifTag::StripOffsets(vec![512], vec![preview]));
        src.set_tag(ExifTag::StripByteCounts(vec![4_000_000]));

        let exif = sanitized(&src, true, None);
        if exif.tiff.len() > 64_000 {
            panic!(
                "embedded preview leaked into exif: {} bytes",
                exif.tiff.len()
            );
        }
        let found = ids(&parse_tiff(&exif.tiff).expect("block parses"));
        if !found.contains(&0x010f) || found.contains(&0x0111) || found.contains(&0x0117) {
            panic!("expected Make without strip tags, got {found:04x?}");
        }
    }
}
