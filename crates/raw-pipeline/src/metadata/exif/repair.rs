use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::exif_tag_format::ExifTagFormat;
use little_exif::ifd::ExifTagGroup;

const ENTRY: usize = 12;
const MAX_IFDS: usize = 16;
const EXIF_POINTER: u16 = 0x8769;
const GPS_POINTER: u16 = 0x8825;
const INTEROP_POINTER: u16 = 0xa005;
const GPS_ALTITUDE_REF: u16 = 0x0005;

enum Verdict {
    Keep,
    Retype(ExifTagFormat),
    Child(usize, ExifTagGroup),
    Drop,
}

struct Tiff {
    bytes: Vec<u8>,
    endian: Endian,
}

pub(super) fn repair(tiff: &[u8]) -> Option<Vec<u8>> {
    let endian = match tiff.get(..2)? {
        b"II" => Endian::Little,
        b"MM" => Endian::Big,
        _ => return None,
    };
    let mut tiff = Tiff {
        bytes: tiff.to_vec(),
        endian,
    };
    let mut pending = vec![(tiff.u32_at(4)? as usize, ExifTagGroup::GENERIC, true)];
    let mut seen: HashSet<usize> = HashSet::new();
    while let Some((ifd, group, chained)) = pending.pop() {
        if seen.len() == MAX_IFDS || !seen.insert(ifd) {
            continue;
        }
        let Some(count) = tiff.fit(ifd) else {
            continue;
        };
        let mut kept = count;
        for index in (0..count).rev() {
            let entry = ifd + 2 + ENTRY * index;
            match tiff.check(entry, &group) {
                Verdict::Keep => {}
                Verdict::Retype(format) => tiff.set_u16(entry + 2, format.as_u16()),
                Verdict::Child(offset, child) => {
                    tiff.set_u16(entry + 2, ExifTagFormat::INT32U.as_u16());
                    pending.push((offset, child, false));
                }
                Verdict::Drop => {
                    tiff.remove(ifd, index, kept);
                    kept -= 1;
                }
            }
        }
        if chained && let Some(next) = tiff.u32_at(ifd + 2 + ENTRY * kept).filter(|&n| n != 0) {
            pending.push((next as usize, ExifTagGroup::GENERIC, true));
        }
    }
    Some(tiff.bytes)
}

fn sub_ifd(group: &ExifTagGroup, tag: u16) -> Option<ExifTagGroup> {
    match (group, tag) {
        (ExifTagGroup::GENERIC, EXIF_POINTER) => Some(ExifTagGroup::EXIF),
        (ExifTagGroup::GENERIC, GPS_POINTER) => Some(ExifTagGroup::GPS),
        (ExifTagGroup::EXIF, INTEROP_POINTER) => Some(ExifTagGroup::INTEROP),
        _ => None,
    }
}

fn family(format: ExifTagFormat) -> Option<u8> {
    match format {
        ExifTagFormat::INT8U
        | ExifTagFormat::INT8S
        | ExifTagFormat::UNDEF
        | ExifTagFormat::STRING => Some(1),
        ExifTagFormat::INT16U | ExifTagFormat::INT16S => Some(2),
        ExifTagFormat::INT32U | ExifTagFormat::INT32S => Some(4),
        ExifTagFormat::RATIONAL64U | ExifTagFormat::RATIONAL64S => Some(8),
        ExifTagFormat::FLOAT | ExifTagFormat::DOUBLE => None,
    }
}

impl Tiff {
    fn u16_at(&self, pos: usize) -> Option<u16> {
        let bytes: [u8; 2] = self.bytes.get(pos..pos.checked_add(2)?)?.try_into().ok()?;
        Some(match self.endian {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        })
    }

    fn u32_at(&self, pos: usize) -> Option<u32> {
        let bytes: [u8; 4] = self.bytes.get(pos..pos.checked_add(4)?)?.try_into().ok()?;
        Some(match self.endian {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        })
    }

    fn set_u16(&mut self, pos: usize, value: u16) {
        let bytes = match self.endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        self.bytes[pos..pos + 2].copy_from_slice(&bytes);
    }

    fn fit(&mut self, ifd: usize) -> Option<usize> {
        let count = self.u16_at(ifd)? as usize;
        let room = self.bytes.len().checked_sub(ifd + 2 + 4)? / ENTRY;
        if count > room {
            self.set_u16(ifd, room as u16);
        }
        Some(count.min(room))
    }

    fn remove(&mut self, ifd: usize, index: usize, count: usize) {
        let start = ifd + 2 + ENTRY * index;
        let end = (ifd + 2 + ENTRY * count + 4).min(self.bytes.len());
        self.bytes.copy_within(start + ENTRY..end, start);
        self.set_u16(ifd, (count - 1) as u16);
    }

    fn data(&self, entry: usize, format: ExifTagFormat, count: u32) -> Option<&[u8]> {
        let len = format.bytes_per_component().checked_mul(count)? as usize;
        let start = if len <= 4 {
            entry + 8
        } else {
            self.u32_at(entry + 8)? as usize
        };
        self.bytes.get(start..start.checked_add(len)?)
    }

    fn check(&self, entry: usize, group: &ExifTagGroup) -> Verdict {
        let (Some(tag), Some(code), Some(count)) = (
            self.u16_at(entry),
            self.u16_at(entry + 2),
            self.u32_at(entry + 4),
        ) else {
            return Verdict::Drop;
        };
        if let Some(child) = sub_ifd(group, tag) {
            return match self.u32_at(entry + 8).map(|offset| offset as usize) {
                Some(offset) if offset + 2 <= self.bytes.len() => Verdict::Child(offset, child),
                _ => Verdict::Drop,
            };
        }
        let Some(format) = ExifTagFormat::from_u16(code) else {
            return Verdict::Drop;
        };
        let Some(data) = self.data(entry, format, count) else {
            return Verdict::Drop;
        };
        let target = match ExifTag::from_u16(tag, group).ok().map(|t| t.format()) {
            None => format,
            Some(expected) if expected == format => format,
            Some(expected) if self.converted(expected, format, group, tag, data) => {
                return Verdict::Keep;
            }
            Some(expected) if family(expected).is_some() && family(expected) == family(format) => {
                expected
            }
            Some(_) => return Verdict::Drop,
        };
        let valid = catch_unwind(AssertUnwindSafe(|| {
            ExifTag::from_u16_with_data(tag, &target, &data.to_vec(), &self.endian, group).is_ok()
        }))
        .unwrap_or(false);
        match (valid, target == format) {
            (false, _) => Verdict::Drop,
            (true, true) => Verdict::Keep,
            (true, false) => Verdict::Retype(target),
        }
    }

    fn converted(
        &self,
        expected: ExifTagFormat,
        found: ExifTagFormat,
        group: &ExifTagGroup,
        tag: u16,
        data: &[u8],
    ) -> bool {
        match (expected, found) {
            (ExifTagFormat::INT32U, ExifTagFormat::INT16U | ExifTagFormat::INT8U)
            | (ExifTagFormat::INT16U, ExifTagFormat::INT32U | ExifTagFormat::INT8U)
            | (ExifTagFormat::RATIONAL64S, ExifTagFormat::RATIONAL64U) => true,
            (ExifTagFormat::INT8U, ExifTagFormat::INT16U) => {
                let high = match self.endian {
                    Endian::Little => 1,
                    Endian::Big => 0,
                };
                data.chunks_exact(2).all(|pair| pair[high] == 0)
            }
            (ExifTagFormat::INT8U, ExifTagFormat::STRING) => {
                *group == ExifTagGroup::GPS
                    && tag == GPS_ALTITUDE_REF
                    && matches!(data.first(), Some(0x00 | 0x01 | 0x30 | 0x31))
            }
            _ => false,
        }
    }
}
