use super::parse::*;
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
fn excessive_huesat_dimensions_are_ignored() {
    let tiff = build_tiff(vec![
        matrix_tag(T_COLOR_MATRIX1, CM1),
        TagVal {
            tag: T_HUESAT_DIMS,
            typ: 4,
            count: 3,
            bytes: longs(&[DCP_MAX_TABLE_DIM + 1, 1, 1]),
        },
        TagVal {
            tag: T_HUESAT_DATA1,
            typ: 11,
            count: 3,
            bytes: floats(&[0.0, 1.0, 1.0]),
        },
    ]);
    let profile = parse_dcp(&tiff).unwrap();
    assert!(profile.huesatmap1.is_none());
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
