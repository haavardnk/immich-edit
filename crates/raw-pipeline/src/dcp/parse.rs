use super::{
    DCP_MAX_SOURCE_BYTES, DCP_MAX_TABLE_DIM, DCP_MAX_TABLE_ENTRIES, DcpParseError, DcpProfile,
    HsvEncoding, HueSatMap,
};
use std::collections::HashMap;
use std::sync::Arc;

const DCP_RT_MAGIC: u16 = 0x4352;

pub(super) const T_UNIQUE_CAMERA_MODEL: u16 = 50708;
pub(super) const T_COLOR_MATRIX1: u16 = 50721;
pub(super) const T_COLOR_MATRIX2: u16 = 50722;
pub(super) const T_CALIB_ILLUM1: u16 = 50778;
pub(super) const T_CALIB_ILLUM2: u16 = 50779;
pub(super) const T_PROFILE_NAME: u16 = 50936;
pub(super) const T_HUESAT_DIMS: u16 = 50937;
pub(super) const T_HUESAT_DATA1: u16 = 50938;
pub(super) const T_HUESAT_DATA2: u16 = 50939;
pub(super) const T_TONE_CURVE: u16 = 50940;
pub(super) const T_EMBED_POLICY: u16 = 50941;
pub(super) const T_COPYRIGHT: u16 = 50942;
pub(super) const T_FORWARD_MATRIX1: u16 = 50964;
pub(super) const T_FORWARD_MATRIX2: u16 = 50965;
pub(super) const T_LOOK_DIMS: u16 = 50981;
pub(super) const T_LOOK_DATA: u16 = 50982;
pub(super) const T_HUESAT_ENCODING: u16 = 51107;
pub(super) const T_LOOK_ENCODING: u16 = 51108;
pub(super) const T_BASELINE_EXPOSURE_OFFSET: u16 = 51109;
pub(super) const T_DEFAULT_BLACK_RENDER: u16 = 51110;

pub fn parse_dcp(bytes: &[u8]) -> Result<DcpProfile, DcpParseError> {
    if bytes.len() > DCP_MAX_SOURCE_BYTES {
        return Err(DcpParseError::TooLarge);
    }
    let tiff = Tiff::new(bytes)?;
    let entries = tiff.entries(tiff.ifd0_off)?;
    let mut map: HashMap<u16, Entry> = HashMap::with_capacity(entries.len());
    for e in entries {
        map.entry(e.tag).or_insert(e);
    }

    let color_matrix1 = map
        .get(&T_COLOR_MATRIX1)
        .and_then(|e| read_matrix3(&tiff, e))
        .ok_or(DcpParseError::MissingColorMatrix)?;

    let ascii = |tag: u16| map.get(&tag).and_then(|e| tiff.ascii(e));
    let u32_at = |tag: u16| {
        map.get(&tag)
            .and_then(|e| tiff.u32_vec(e))
            .and_then(|v| v.first().copied())
    };

    Ok(DcpProfile {
        name: ascii(T_PROFILE_NAME),
        copyright: ascii(T_COPYRIGHT),
        unique_camera_model: ascii(T_UNIQUE_CAMERA_MODEL),
        calibration_illuminant1: u32_at(T_CALIB_ILLUM1).unwrap_or(21) as u16,
        calibration_illuminant2: u32_at(T_CALIB_ILLUM2).map(|v| v as u16),
        color_matrix1,
        color_matrix2: map
            .get(&T_COLOR_MATRIX2)
            .and_then(|e| read_matrix3(&tiff, e)),
        forward_matrix1: map
            .get(&T_FORWARD_MATRIX1)
            .and_then(|e| read_matrix3(&tiff, e)),
        forward_matrix2: map
            .get(&T_FORWARD_MATRIX2)
            .and_then(|e| read_matrix3(&tiff, e)),
        huesatmap1: read_huesat(
            &tiff,
            map.get(&T_HUESAT_DIMS),
            map.get(&T_HUESAT_DATA1),
            map.get(&T_HUESAT_ENCODING),
        )
        .map(Arc::new),
        huesatmap2: read_huesat(
            &tiff,
            map.get(&T_HUESAT_DIMS),
            map.get(&T_HUESAT_DATA2),
            map.get(&T_HUESAT_ENCODING),
        )
        .map(Arc::new),
        look_table: read_huesat(
            &tiff,
            map.get(&T_LOOK_DIMS),
            map.get(&T_LOOK_DATA),
            map.get(&T_LOOK_ENCODING),
        )
        .map(Arc::new),
        tone_curve: read_tone_curve(&tiff, map.get(&T_TONE_CURVE)).map(Arc::new),
        baseline_exposure_offset: map
            .get(&T_BASELINE_EXPOSURE_OFFSET)
            .and_then(|e| tiff.f32_vec(e))
            .and_then(|v| v.first().copied())
            .unwrap_or(0.0),
        default_black_render: u32_at(T_DEFAULT_BLACK_RENDER).unwrap_or(0),
        embed_policy: u32_at(T_EMBED_POLICY).unwrap_or(0),
    })
}

fn read_matrix3(tiff: &Tiff, e: &Entry) -> Option<[[f32; 3]; 3]> {
    let v = tiff.f32_vec(e)?;
    if v.len() < 9 || v.iter().any(|x| !x.is_finite()) {
        return None;
    }
    Some([[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]])
}

fn read_huesat(
    tiff: &Tiff,
    dims: Option<&Entry>,
    data: Option<&Entry>,
    encoding: Option<&Entry>,
) -> Option<HueSatMap> {
    let dims = tiff.u32_vec(dims?)?;
    if dims.len() < 3 {
        return None;
    }
    let raw = tiff.f32_vec(data?)?;
    if raw.len() % 3 != 0 || raw.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let encoding = encoding
        .and_then(|e| tiff.u32_vec(e))
        .and_then(|v| v.first().copied())
        .map(HsvEncoding::from_code)
        .unwrap_or(HsvEncoding::Linear);
    let hue = dims[0] as usize;
    let sat = dims[1] as usize;
    let val = dims[2].max(1) as usize;
    if dims[0] == 0
        || dims[1] == 0
        || dims[0] > DCP_MAX_TABLE_DIM
        || dims[1] > DCP_MAX_TABLE_DIM
        || dims[2].max(1) > DCP_MAX_TABLE_DIM
    {
        return None;
    }
    let expected = hue.checked_mul(sat)?.checked_mul(val)?;
    if expected > DCP_MAX_TABLE_ENTRIES {
        return None;
    }
    let file: Vec<[f32; 3]> = raw.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    if file.len() != expected {
        return None;
    }
    let mut data = vec![[0.0f32; 3]; file.len()];
    for (src, px) in file.into_iter().enumerate() {
        let s = src % sat;
        let hv = src / sat;
        let h = hv % hue;
        let v = hv / hue;
        data[(v * sat + s) * hue + h] = px;
    }
    let map = HueSatMap {
        hue_div: dims[0],
        sat_div: dims[1],
        val_div: dims[2].max(1),
        encoding,
        data,
    };
    if map.data.len() != map.expected_len()? {
        return None;
    }
    Some(map)
}

fn read_tone_curve(tiff: &Tiff, e: Option<&Entry>) -> Option<Vec<[f32; 2]>> {
    let raw = tiff.f32_vec(e?)?;
    if raw.len() < 4 || raw.len() % 2 != 0 || raw.iter().any(|x| !x.is_finite()) {
        return None;
    }
    Some(raw.chunks_exact(2).map(|c| [c[0], c[1]]).collect())
}

struct Entry {
    tag: u16,
    typ: u16,
    count: u32,
    value_field: usize,
}

struct Tiff<'a> {
    data: &'a [u8],
    le: bool,
    ifd0_off: usize,
}

impl<'a> Tiff<'a> {
    fn new(data: &'a [u8]) -> Result<Self, DcpParseError> {
        if data.len() < 8 {
            return Err(DcpParseError::NotTiff);
        }
        let le = match &data[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => return Err(DcpParseError::NotTiff),
        };
        let magic = read_u16(data, 2, le);
        if magic != 42 && magic != DCP_RT_MAGIC {
            return Err(DcpParseError::NotTiff);
        }
        let ifd0_off = read_u32(data, 4, le) as usize;
        if ifd0_off + 2 > data.len() {
            return Err(DcpParseError::Truncated);
        }
        Ok(Self { data, le, ifd0_off })
    }

    fn entries(&self, ifd_off: usize) -> Result<Vec<Entry>, DcpParseError> {
        if ifd_off + 2 > self.data.len() {
            return Err(DcpParseError::Truncated);
        }
        let count = read_u16(self.data, ifd_off, self.le) as usize;
        let end = ifd_off + 2 + count * 12;
        if end > self.data.len() {
            return Err(DcpParseError::Truncated);
        }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let base = ifd_off + 2 + i * 12;
            out.push(Entry {
                tag: read_u16(self.data, base, self.le),
                typ: read_u16(self.data, base + 2, self.le),
                count: read_u32(self.data, base + 4, self.le),
                value_field: base + 8,
            });
        }
        Ok(out)
    }

    fn value_base(&self, e: &Entry) -> Option<usize> {
        let elem = type_size(e.typ)?;
        let total = elem.checked_mul(e.count as usize)?;
        let base = if total <= 4 {
            e.value_field
        } else {
            read_u32(self.data, e.value_field, self.le) as usize
        };
        if base + total > self.data.len() {
            return None;
        }
        Some(base)
    }

    fn ascii(&self, e: &Entry) -> Option<String> {
        if e.typ != 2 {
            return None;
        }
        let base = self.value_base(e)?;
        let raw = &self.data[base..base + e.count as usize];
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        Some(String::from_utf8_lossy(&raw[..end]).trim().to_string())
    }

    fn u32_vec(&self, e: &Entry) -> Option<Vec<u32>> {
        let base = self.value_base(e)?;
        let n = e.count as usize;
        let out = match e.typ {
            3 => (0..n)
                .map(|i| read_u16(self.data, base + i * 2, self.le) as u32)
                .collect(),
            4 => (0..n)
                .map(|i| read_u32(self.data, base + i * 4, self.le))
                .collect(),
            _ => return None,
        };
        Some(out)
    }

    fn f32_vec(&self, e: &Entry) -> Option<Vec<f32>> {
        let base = self.value_base(e)?;
        let n = e.count as usize;
        let out = match e.typ {
            3 => (0..n)
                .map(|i| read_u16(self.data, base + i * 2, self.le) as f32)
                .collect(),
            4 => (0..n)
                .map(|i| read_u32(self.data, base + i * 4, self.le) as f32)
                .collect(),
            5 => (0..n)
                .map(|i| {
                    let num = read_u32(self.data, base + i * 8, self.le) as f32;
                    let den = read_u32(self.data, base + i * 8 + 4, self.le) as f32;
                    if den == 0.0 { 0.0 } else { num / den }
                })
                .collect(),
            10 => (0..n)
                .map(|i| {
                    let num = read_i32(self.data, base + i * 8, self.le) as f32;
                    let den = read_i32(self.data, base + i * 8 + 4, self.le) as f32;
                    if den == 0.0 { 0.0 } else { num / den }
                })
                .collect(),
            11 => (0..n)
                .map(|i| f32::from_bits(read_u32(self.data, base + i * 4, self.le)))
                .collect(),
            _ => return None,
        };
        Some(out)
    }
}

fn type_size(typ: u16) -> Option<usize> {
    match typ {
        1 | 2 | 6 | 7 => Some(1),
        3 | 8 => Some(2),
        4 | 9 | 11 => Some(4),
        5 | 10 | 12 => Some(8),
        _ => None,
    }
}

fn read_u16(d: &[u8], off: usize, le: bool) -> u16 {
    let b = [d[off], d[off + 1]];
    if le {
        u16::from_le_bytes(b)
    } else {
        u16::from_be_bytes(b)
    }
}

fn read_u32(d: &[u8], off: usize, le: bool) -> u32 {
    let b = [d[off], d[off + 1], d[off + 2], d[off + 3]];
    if le {
        u32::from_le_bytes(b)
    } else {
        u32::from_be_bytes(b)
    }
}

fn read_i32(d: &[u8], off: usize, le: bool) -> i32 {
    let b = [d[off], d[off + 1], d[off + 2], d[off + 3]];
    if le {
        i32::from_le_bytes(b)
    } else {
        i32::from_be_bytes(b)
    }
}
