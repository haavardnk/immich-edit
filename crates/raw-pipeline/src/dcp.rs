use std::collections::HashMap;
use std::sync::Arc;

pub const DCP_MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

pub type DcpMap = HashMap<String, Arc<DcpProfile>>;

pub fn empty_dcp() -> DcpMap {
    HashMap::new()
}

#[derive(Debug, Clone, PartialEq)]
pub enum DcpParseError {
    TooLarge,
    NotTiff,
    Truncated,
    MissingColorMatrix,
    Invalid(&'static str),
}

impl std::fmt::Display for DcpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "dcp source exceeds size limit"),
            Self::NotTiff => write!(f, "not a TIFF/DCP container"),
            Self::Truncated => write!(f, "truncated dcp data"),
            Self::MissingColorMatrix => write!(f, "missing ColorMatrix1"),
            Self::Invalid(m) => write!(f, "invalid dcp: {m}"),
        }
    }
}

impl std::error::Error for DcpParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsvEncoding {
    Linear,
    Srgb,
}

impl HsvEncoding {
    fn from_code(code: u32) -> Self {
        if code == 1 { Self::Srgb } else { Self::Linear }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcpIlluminant {
    #[default]
    Interpolated,
    First,
    Second,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HueSatMap {
    pub hue_div: u32,
    pub sat_div: u32,
    pub val_div: u32,
    pub encoding: HsvEncoding,
    pub data: Vec<[f32; 3]>,
}

impl HueSatMap {
    fn expected_len(&self) -> usize {
        (self.hue_div * self.sat_div * self.val_div.max(1)) as usize
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DcpProfile {
    pub name: Option<String>,
    pub copyright: Option<String>,
    pub unique_camera_model: Option<String>,
    pub calibration_illuminant1: u16,
    pub calibration_illuminant2: Option<u16>,
    pub color_matrix1: [[f32; 3]; 3],
    pub color_matrix2: Option<[[f32; 3]; 3]>,
    pub forward_matrix1: Option<[[f32; 3]; 3]>,
    pub forward_matrix2: Option<[[f32; 3]; 3]>,
    pub huesatmap1: Option<HueSatMap>,
    pub huesatmap2: Option<HueSatMap>,
    pub look_table: Option<HueSatMap>,
    pub tone_curve: Option<Vec<[f32; 2]>>,
    pub baseline_exposure_offset: f32,
    pub default_black_render: u32,
    pub embed_policy: u32,
}

impl DcpProfile {
    pub fn is_dual_illuminant(&self) -> bool {
        self.calibration_illuminant2.is_some() && self.color_matrix2.is_some()
    }

    pub fn has_tone_curve(&self) -> bool {
        self.tone_curve.as_ref().is_some_and(|c| c.len() >= 2)
    }

    pub fn has_base_table(&self) -> bool {
        self.huesatmap1.is_some()
    }

    pub fn has_look_table(&self) -> bool {
        self.look_table.is_some()
    }

    pub fn has_forward_matrix(&self) -> bool {
        self.forward_matrix1.is_some()
    }

    pub fn is_adobe(&self) -> bool {
        self.copyright
            .as_deref()
            .is_some_and(|c| c.contains("Adobe"))
    }
}

const DCP_RT_MAGIC: u16 = 0x4352;

const T_UNIQUE_CAMERA_MODEL: u16 = 50708;
const T_COLOR_MATRIX1: u16 = 50721;
const T_COLOR_MATRIX2: u16 = 50722;
const T_CALIB_ILLUM1: u16 = 50778;
const T_CALIB_ILLUM2: u16 = 50779;
const T_PROFILE_NAME: u16 = 50936;
const T_HUESAT_DIMS: u16 = 50937;
const T_HUESAT_DATA1: u16 = 50938;
const T_HUESAT_DATA2: u16 = 50939;
const T_TONE_CURVE: u16 = 50940;
const T_EMBED_POLICY: u16 = 50941;
const T_COPYRIGHT: u16 = 50942;
const T_FORWARD_MATRIX1: u16 = 50964;
const T_FORWARD_MATRIX2: u16 = 50965;
const T_LOOK_DIMS: u16 = 50981;
const T_LOOK_DATA: u16 = 50982;
const T_HUESAT_ENCODING: u16 = 51107;
const T_LOOK_ENCODING: u16 = 51108;
const T_BASELINE_EXPOSURE_OFFSET: u16 = 51109;
const T_DEFAULT_BLACK_RENDER: u16 = 51110;

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
        ),
        huesatmap2: read_huesat(
            &tiff,
            map.get(&T_HUESAT_DIMS),
            map.get(&T_HUESAT_DATA2),
            map.get(&T_HUESAT_ENCODING),
        ),
        look_table: read_huesat(
            &tiff,
            map.get(&T_LOOK_DIMS),
            map.get(&T_LOOK_DATA),
            map.get(&T_LOOK_ENCODING),
        ),
        tone_curve: read_tone_curve(&tiff, map.get(&T_TONE_CURVE)),
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
    let file: Vec<[f32; 3]> = raw.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    if hue == 0 || sat == 0 || file.len() != hue * sat * val {
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
    if map.hue_div == 0 || map.sat_div == 0 || map.data.len() != map.expected_len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TagVal {
        tag: u16,
        typ: u16,
        count: u32,
        bytes: Vec<u8>,
    }

    fn srational(vals: &[f32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(vals.len() * 8);
        for v in vals {
            let num = (v * 10_000.0).round() as i32;
            out.extend_from_slice(&num.to_le_bytes());
            out.extend_from_slice(&10_000i32.to_le_bytes());
        }
        out
    }

    fn floats(vals: &[f32]) -> Vec<u8> {
        vals.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn longs(vals: &[u32]) -> Vec<u8> {
        vals.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn short(val: u16) -> Vec<u8> {
        val.to_le_bytes().to_vec()
    }

    fn ascii(s: &str) -> Vec<u8> {
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        b
    }

    fn matrix_tag(tag: u16, m: [[f32; 3]; 3]) -> TagVal {
        let flat: Vec<f32> = m.iter().flatten().copied().collect();
        TagVal {
            tag,
            typ: 10,
            count: 9,
            bytes: srational(&flat),
        }
    }

    fn build_tiff(mut entries: Vec<TagVal>) -> Vec<u8> {
        entries.sort_by_key(|e| e.tag);
        let n = entries.len();
        let ifd_off = 8usize;
        let data_off = ifd_off + 2 + n * 12 + 4;
        let mut out = Vec::new();
        out.extend_from_slice(b"II");
        out.extend_from_slice(&42u16.to_le_bytes());
        out.extend_from_slice(&(ifd_off as u32).to_le_bytes());
        out.extend_from_slice(&(n as u16).to_le_bytes());

        let mut blob = Vec::new();
        for e in &entries {
            out.extend_from_slice(&e.tag.to_le_bytes());
            out.extend_from_slice(&e.typ.to_le_bytes());
            out.extend_from_slice(&e.count.to_le_bytes());
            if e.bytes.len() <= 4 {
                let mut field = e.bytes.clone();
                field.resize(4, 0);
                out.extend_from_slice(&field);
            } else {
                let off = data_off + blob.len();
                out.extend_from_slice(&(off as u32).to_le_bytes());
                blob.extend_from_slice(&e.bytes);
            }
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&blob);
        out
    }

    const CM1: [[f32; 3]; 3] = [[0.6, -0.1, -0.05], [-0.3, 1.2, 0.1], [0.02, -0.2, 0.9]];

    #[test]
    fn parses_minimal_color_matrix() {
        let tiff = build_tiff(vec![matrix_tag(T_COLOR_MATRIX1, CM1)]);
        let p = parse_dcp(&tiff).unwrap();
        for (row, expected) in p.color_matrix1.iter().zip(CM1.iter()) {
            for (got, want) in row.iter().zip(expected.iter()) {
                assert!((got - want).abs() < 1e-3);
            }
        }
        assert!(!p.is_dual_illuminant());
        assert!(!p.has_forward_matrix());
        assert!(p.name.is_none());
    }

    #[test]
    fn parses_rawtherapee_magic() {
        let mut tiff = build_tiff(vec![matrix_tag(T_COLOR_MATRIX1, CM1)]);
        tiff[2] = 0x52;
        tiff[3] = 0x43;
        let p = parse_dcp(&tiff).unwrap();
        for (row, expected) in p.color_matrix1.iter().zip(CM1.iter()) {
            for (got, want) in row.iter().zip(expected.iter()) {
                assert!((got - want).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn parses_dual_illuminant_profile() {
        let fm: [[f32; 3]; 3] = [[0.4, 0.35, 0.15], [0.2, 0.7, 0.06], [0.02, 0.1, 0.8]];
        let tiff = build_tiff(vec![
            matrix_tag(T_COLOR_MATRIX1, CM1),
            matrix_tag(T_COLOR_MATRIX2, CM1),
            matrix_tag(T_FORWARD_MATRIX1, fm),
            matrix_tag(T_FORWARD_MATRIX2, fm),
            TagVal {
                tag: T_CALIB_ILLUM1,
                typ: 3,
                count: 1,
                bytes: short(17),
            },
            TagVal {
                tag: T_CALIB_ILLUM2,
                typ: 3,
                count: 1,
                bytes: short(21),
            },
            TagVal {
                tag: T_PROFILE_NAME,
                typ: 2,
                count: 9,
                bytes: ascii("Test Cam"),
            },
            TagVal {
                tag: T_COPYRIGHT,
                typ: 2,
                count: 16,
                bytes: ascii("RawTherapee CC0"),
            },
        ]);
        let p = parse_dcp(&tiff).unwrap();
        assert!(p.is_dual_illuminant());
        assert!(p.has_forward_matrix());
        assert!(!p.is_adobe());
        assert_eq!(p.calibration_illuminant1, 17);
        assert_eq!(p.calibration_illuminant2, Some(21));
        assert_eq!(p.name.as_deref(), Some("Test Cam"));
        assert_eq!(p.copyright.as_deref(), Some("RawTherapee CC0"));
    }

    #[test]
    fn parses_huesat_and_tone_curve() {
        let hsv = floats(&[1.0, 1.0, 1.0, 2.0, 1.1, 1.0, -1.0, 0.9, 1.0, 0.0, 1.0, 1.0]);
        let tiff = build_tiff(vec![
            matrix_tag(T_COLOR_MATRIX1, CM1),
            TagVal {
                tag: T_HUESAT_DIMS,
                typ: 4,
                count: 3,
                bytes: longs(&[2, 2, 1]),
            },
            TagVal {
                tag: T_HUESAT_DATA1,
                typ: 11,
                count: 12,
                bytes: hsv,
            },
            TagVal {
                tag: T_HUESAT_ENCODING,
                typ: 4,
                count: 1,
                bytes: longs(&[1]),
            },
            TagVal {
                tag: T_TONE_CURVE,
                typ: 11,
                count: 6,
                bytes: floats(&[0.0, 0.0, 0.5, 0.6, 1.0, 1.0]),
            },
        ]);
        let p = parse_dcp(&tiff).unwrap();
        let hs = p.huesatmap1.as_ref().expect("huesat");
        assert_eq!((hs.hue_div, hs.sat_div, hs.val_div), (2, 2, 1));
        assert_eq!(hs.encoding, HsvEncoding::Srgb);
        assert_eq!(hs.data.len(), 4);
        assert!(p.has_tone_curve());
        assert_eq!(p.tone_curve.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn huesat_data_reordered_to_hue_fastest() {
        let hsv = floats(&[0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 10.0, 1.0, 1.0, 11.0, 1.0, 1.0]);
        let tiff = build_tiff(vec![
            matrix_tag(T_COLOR_MATRIX1, CM1),
            TagVal {
                tag: T_HUESAT_DIMS,
                typ: 4,
                count: 3,
                bytes: longs(&[2, 2, 1]),
            },
            TagVal {
                tag: T_HUESAT_DATA1,
                typ: 11,
                count: 12,
                bytes: hsv,
            },
        ]);
        let p = parse_dcp(&tiff).unwrap();
        let hs = p.huesatmap1.as_ref().expect("huesat");
        let shifts: Vec<f32> = hs.data.iter().map(|d| d[0]).collect();
        assert_eq!(shifts, vec![0.0, 10.0, 1.0, 11.0]);
    }

    #[test]
    fn rejects_non_tiff() {
        assert_eq!(
            parse_dcp(b"not a tiff at all").unwrap_err(),
            DcpParseError::NotTiff
        );
    }

    #[test]
    fn requires_color_matrix() {
        let tiff = build_tiff(vec![TagVal {
            tag: T_PROFILE_NAME,
            typ: 2,
            count: 5,
            bytes: ascii("x"),
        }]);
        assert_eq!(
            parse_dcp(&tiff).unwrap_err(),
            DcpParseError::MissingColorMatrix
        );
    }
}
