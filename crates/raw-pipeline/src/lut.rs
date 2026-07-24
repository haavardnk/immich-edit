use std::collections::HashMap;
use std::sync::Arc;

pub const LUT_MIN_SIZE: usize = 2;
pub const LUT_MAX_SIZE: usize = 65;
pub const LUT_MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LutParseError {
    TooLarge,
    Empty,
    MissingSize,
    DuplicateDirective(&'static str),
    InvalidSize,
    UnsupportedOneDimensional,
    UnknownDirective(String),
    InvalidNumber,
    NonFiniteValue,
    WrongEntryCount { expected: usize, found: usize },
    InvalidDomain,
}

impl std::fmt::Display for LutParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "cube source exceeds size limit"),
            Self::Empty => write!(f, "cube source is empty"),
            Self::MissingSize => write!(f, "missing LUT_3D_SIZE"),
            Self::DuplicateDirective(d) => write!(f, "duplicate directive {d}"),
            Self::InvalidSize => write!(f, "LUT_3D_SIZE out of range"),
            Self::UnsupportedOneDimensional => write!(f, "1D LUTs are not supported"),
            Self::UnknownDirective(d) => write!(f, "unknown directive {d}"),
            Self::InvalidNumber => write!(f, "invalid numeric value"),
            Self::NonFiniteValue => write!(f, "non-finite value"),
            Self::WrongEntryCount { expected, found } => {
                write!(f, "expected {expected} entries, found {found}")
            }
            Self::InvalidDomain => write!(f, "invalid domain range"),
        }
    }
}

impl std::error::Error for LutParseError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Lut3d {
    size: usize,
    domain_min: [f32; 3],
    domain_max: [f32; 3],
    data: Vec<[f32; 3]>,
}

impl Lut3d {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn domain_min(&self) -> [f32; 3] {
        self.domain_min
    }

    pub fn domain_max(&self) -> [f32; 3] {
        self.domain_max
    }

    pub fn data(&self) -> &[[f32; 3]] {
        &self.data
    }

    pub fn parse_cube(source: &[u8]) -> Result<Self, LutParseError> {
        if source.len() > LUT_MAX_SOURCE_BYTES {
            return Err(LutParseError::TooLarge);
        }
        let text = String::from_utf8_lossy(source);
        let mut size: Option<usize> = None;
        let mut domain_min: Option<[f32; 3]> = None;
        let mut domain_max: Option<[f32; 3]> = None;
        let mut title_seen = false;
        let mut data: Vec<[f32; 3]> = Vec::new();

        for raw_line in text.lines() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let keyword = parts.next().unwrap_or("");
            match keyword {
                "TITLE" => {
                    if title_seen {
                        return Err(LutParseError::DuplicateDirective("TITLE"));
                    }
                    title_seen = true;
                }
                "LUT_3D_SIZE" => {
                    if size.is_some() {
                        return Err(LutParseError::DuplicateDirective("LUT_3D_SIZE"));
                    }
                    let n: usize = parts
                        .next()
                        .ok_or(LutParseError::InvalidSize)?
                        .parse()
                        .map_err(|_| LutParseError::InvalidSize)?;
                    if !(LUT_MIN_SIZE..=LUT_MAX_SIZE).contains(&n) {
                        return Err(LutParseError::InvalidSize);
                    }
                    size = Some(n);
                }
                "LUT_1D_SIZE" => {
                    return Err(LutParseError::UnsupportedOneDimensional);
                }
                "DOMAIN_MIN" => {
                    if domain_min.is_some() {
                        return Err(LutParseError::DuplicateDirective("DOMAIN_MIN"));
                    }
                    domain_min = Some(parse_triple(&mut parts)?);
                }
                "DOMAIN_MAX" => {
                    if domain_max.is_some() {
                        return Err(LutParseError::DuplicateDirective("DOMAIN_MAX"));
                    }
                    domain_max = Some(parse_triple(&mut parts)?);
                }
                _ => {
                    let r: f32 = parse_number(keyword)?;
                    let g: f32 = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
                    let b: f32 = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
                    if parts.next().is_some() {
                        return Err(LutParseError::InvalidNumber);
                    }
                    data.push([r, g, b]);
                }
            }
        }

        let size = size.ok_or(LutParseError::MissingSize)?;
        let expected = size * size * size;
        if data.is_empty() {
            return Err(LutParseError::Empty);
        }
        if data.len() != expected {
            return Err(LutParseError::WrongEntryCount {
                expected,
                found: data.len(),
            });
        }
        let domain_min = domain_min.unwrap_or([0.0, 0.0, 0.0]);
        let domain_max = domain_max.unwrap_or([1.0, 1.0, 1.0]);
        for c in 0..3 {
            if domain_max[c] - domain_min[c] <= f32::EPSILON {
                return Err(LutParseError::InvalidDomain);
            }
        }

        Ok(Self {
            size,
            domain_min,
            domain_max,
            data,
        })
    }

    #[inline]
    fn at(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        self.data[(b * self.size + g) * self.size + r]
    }

    pub fn sample(&self, rgb: [f32; 3]) -> [f32; 3] {
        let n = self.size;
        let last = (n - 1) as f32;
        let mut coord = [0.0f32; 3];
        for c in 0..3 {
            let span = self.domain_max[c] - self.domain_min[c];
            let normalized = ((rgb[c] - self.domain_min[c]) / span).clamp(0.0, 1.0);
            coord[c] = normalized * last;
        }
        let base = [
            (coord[0].floor() as usize).min(n - 2),
            (coord[1].floor() as usize).min(n - 2),
            (coord[2].floor() as usize).min(n - 2),
        ];
        let hi = [base[0] + 1, base[1] + 1, base[2] + 1];
        let fr = coord[0] - base[0] as f32;
        let fg = coord[1] - base[1] as f32;
        let fb = coord[2] - base[2] as f32;

        let c000 = self.at(base[0], base[1], base[2]);
        let c111 = self.at(hi[0], hi[1], hi[2]);
        let (w0, w1, w2) = tetra_weights(fr, fg, fb);
        let (v1, v2) = tetra_corners(self, base, hi, fr, fg, fb);

        let mut out = [0.0f32; 3];
        for (c, o) in out.iter_mut().enumerate() {
            *o = c000[c] + w0 * (v1[c] - c000[c]) + w1 * (v2[c] - v1[c]) + w2 * (c111[c] - v2[c]);
        }
        out
    }
}

fn tetra_weights(fr: f32, fg: f32, fb: f32) -> (f32, f32, f32) {
    if fr >= fg && fg >= fb {
        (fr, fg, fb)
    } else if fr >= fb && fb >= fg {
        (fr, fb, fg)
    } else if fb >= fr && fr >= fg {
        (fb, fr, fg)
    } else if fg >= fr && fr >= fb {
        (fg, fr, fb)
    } else if fg >= fb && fb >= fr {
        (fg, fb, fr)
    } else {
        (fb, fg, fr)
    }
}

fn tetra_corners(
    lut: &Lut3d,
    base: [usize; 3],
    hi: [usize; 3],
    fr: f32,
    fg: f32,
    fb: f32,
) -> ([f32; 3], [f32; 3]) {
    let (b0, b1, b2) = (base[0], base[1], base[2]);
    let (h0, h1, h2) = (hi[0], hi[1], hi[2]);
    if fr >= fg && fg >= fb {
        (lut.at(h0, b1, b2), lut.at(h0, h1, b2))
    } else if fr >= fb && fb >= fg {
        (lut.at(h0, b1, b2), lut.at(h0, b1, h2))
    } else if fb >= fr && fr >= fg {
        (lut.at(b0, b1, h2), lut.at(h0, b1, h2))
    } else if fg >= fr && fr >= fb {
        (lut.at(b0, h1, b2), lut.at(h0, h1, b2))
    } else if fg >= fb && fb >= fr {
        (lut.at(b0, h1, b2), lut.at(b0, h1, h2))
    } else {
        (lut.at(b0, b1, h2), lut.at(b0, h1, h2))
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }
}

fn parse_number(token: &str) -> Result<f32, LutParseError> {
    let v: f32 = token.parse().map_err(|_| LutParseError::InvalidNumber)?;
    if !v.is_finite() {
        return Err(LutParseError::NonFiniteValue);
    }
    Ok(v)
}

fn parse_triple<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<[f32; 3], LutParseError> {
    let a = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
    let b = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
    let c = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
    if parts.next().is_some() {
        return Err(LutParseError::InvalidNumber);
    }
    Ok([a, b, c])
}

pub type LutMap = HashMap<String, Arc<Lut3d>>;

pub fn empty_luts() -> LutMap {
    LutMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_cube(n: usize) -> String {
        let mut s = format!("LUT_3D_SIZE {n}\n");
        let last = (n - 1) as f32;
        for b in 0..n {
            for g in 0..n {
                for r in 0..n {
                    s.push_str(&format!(
                        "{} {} {}\n",
                        r as f32 / last,
                        g as f32 / last,
                        b as f32 / last
                    ));
                }
            }
        }
        s
    }

    #[test]
    fn parses_identity() {
        let lut = Lut3d::parse_cube(identity_cube(2).as_bytes()).unwrap();
        assert_eq!(lut.size(), 2);
        assert_eq!(lut.data().len(), 8);
    }

    #[test]
    fn identity_lut_is_neutral() {
        let lut = Lut3d::parse_cube(identity_cube(17).as_bytes()).unwrap();
        for &probe in &[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.25, 0.5, 0.75],
            [0.1, 0.9, 0.3],
        ] {
            let out = lut.sample(probe);
            for c in 0..3 {
                assert!(
                    (out[c] - probe[c]).abs() < 1e-4,
                    "channel {c}: {} vs {}",
                    out[c],
                    probe[c]
                );
            }
        }
    }

    #[test]
    fn parses_title_and_domain() {
        let src = format!("TITLE \"x\"\nDOMAIN_MIN 0 0 0\nDOMAIN_MAX 1 1 1\n{}", {
            let mut s = String::from("LUT_3D_SIZE 2\n");
            for b in 0..2 {
                for g in 0..2 {
                    for r in 0..2 {
                        s.push_str(&format!("{} {} {}\n", r, g, b));
                    }
                }
            }
            s
        });
        let lut = Lut3d::parse_cube(src.as_bytes()).unwrap();
        assert_eq!(lut.size(), 2);
    }

    #[test]
    fn rejects_missing_size() {
        assert_eq!(
            Lut3d::parse_cube(b"0 0 0\n1 1 1\n"),
            Err(LutParseError::MissingSize)
        );
    }

    #[test]
    fn rejects_1d() {
        assert_eq!(
            Lut3d::parse_cube(b"LUT_1D_SIZE 2\n0 0 0\n1 1 1\n"),
            Err(LutParseError::UnsupportedOneDimensional)
        );
    }

    #[test]
    fn rejects_wrong_count() {
        let r = Lut3d::parse_cube(b"LUT_3D_SIZE 2\n0 0 0\n1 1 1\n");
        assert!(matches!(r, Err(LutParseError::WrongEntryCount { .. })));
    }

    #[test]
    fn rejects_non_finite() {
        let src = "LUT_3D_SIZE 2\nnan 0 0\n".to_string() + &"0 0 0\n".repeat(7);
        assert_eq!(
            Lut3d::parse_cube(src.as_bytes()),
            Err(LutParseError::NonFiniteValue)
        );
    }

    #[test]
    fn rejects_bad_size() {
        assert_eq!(
            Lut3d::parse_cube(b"LUT_3D_SIZE 1\n0 0 0\n"),
            Err(LutParseError::InvalidSize)
        );
        assert_eq!(
            Lut3d::parse_cube(b"LUT_3D_SIZE 999\n"),
            Err(LutParseError::InvalidSize)
        );
    }

    #[test]
    fn rejects_duplicate_size() {
        let r = Lut3d::parse_cube(b"LUT_3D_SIZE 2\nLUT_3D_SIZE 2\n");
        assert_eq!(r, Err(LutParseError::DuplicateDirective("LUT_3D_SIZE")));
    }

    #[test]
    fn rejects_too_large() {
        let big = vec![b'0'; LUT_MAX_SOURCE_BYTES + 1];
        assert_eq!(Lut3d::parse_cube(&big), Err(LutParseError::TooLarge));
    }

    #[test]
    fn invert_lut_maps_corners() {
        let mut s = String::from("LUT_3D_SIZE 2\n");
        let last = 1.0f32;
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    s.push_str(&format!(
                        "{} {} {}\n",
                        1.0 - r as f32 / last,
                        1.0 - g as f32 / last,
                        1.0 - b as f32 / last
                    ));
                }
            }
        }
        let lut = Lut3d::parse_cube(s.as_bytes()).unwrap();
        let out = lut.sample([0.0, 0.0, 0.0]);
        assert!((out[0] - 1.0).abs() < 1e-4);
        let out2 = lut.sample([1.0, 1.0, 1.0]);
        assert!(out2[0].abs() < 1e-4);
    }

    #[test]
    fn domain_scales_input() {
        let mut s = String::from("DOMAIN_MAX 2 2 2\nLUT_3D_SIZE 2\n");
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    s.push_str(&format!("{} {} {}\n", r, g, b));
                }
            }
        }
        let lut = Lut3d::parse_cube(s.as_bytes()).unwrap();
        let out = lut.sample([1.0, 1.0, 1.0]);
        for (c, v) in out.iter().enumerate() {
            assert!((v - 0.5).abs() < 1e-4, "channel {c} = {v}");
        }
    }
}
