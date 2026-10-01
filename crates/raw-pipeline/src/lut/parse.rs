use super::{
    CubeLut, Domain, LUT_1D_MAX_SIZE, LUT_MAX_SIZE, LUT_MAX_SOURCE_BYTES, LUT_MIN_SIZE, Lut1d,
    Lut3d,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LutParseError {
    TooLarge,
    Empty,
    MissingSize,
    DuplicateDirective(&'static str),
    InvalidSize,
    UnknownDirective(String),
    InvalidNumber,
    NonFiniteValue,
    WrongEntryCount { expected: usize, found: usize },
    InvalidDomain,
    ConflictingDomain,
}

impl std::fmt::Display for LutParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "cube source exceeds size limit"),
            Self::Empty => write!(f, "cube source is empty"),
            Self::MissingSize => write!(f, "missing LUT_1D_SIZE or LUT_3D_SIZE"),
            Self::DuplicateDirective(d) => write!(f, "duplicate directive {d}"),
            Self::InvalidSize => write!(f, "LUT size out of range"),
            Self::UnknownDirective(d) => write!(f, "unknown directive {d}"),
            Self::InvalidNumber => write!(f, "invalid numeric value"),
            Self::NonFiniteValue => write!(f, "non-finite value"),
            Self::WrongEntryCount { expected, found } => {
                write!(f, "expected {expected} entries, found {found}")
            }
            Self::InvalidDomain => write!(f, "invalid domain range"),
            Self::ConflictingDomain => write!(f, "domain directives conflict with the tables"),
        }
    }
}

impl std::error::Error for LutParseError {}

#[derive(Default)]
struct Header {
    title: bool,
    size_1d: Option<usize>,
    size_3d: Option<usize>,
    domain_min: Option<[f32; 3]>,
    domain_max: Option<[f32; 3]>,
    range_1d: Option<[f32; 2]>,
    range_3d: Option<[f32; 2]>,
}

impl Header {
    fn domain(&self) -> Option<Domain> {
        if self.domain_min.is_none() && self.domain_max.is_none() {
            return None;
        }
        Some(Domain {
            min: self.domain_min.unwrap_or(Domain::UNIT.min),
            max: self.domain_max.unwrap_or(Domain::UNIT.max),
        })
    }

    fn domains(&self) -> Result<(Domain, Domain), LutParseError> {
        let both = self.size_1d.is_some() && self.size_3d.is_some();
        let shared = self.domain();
        if shared.is_some() && (both || self.range_1d.is_some() || self.range_3d.is_some()) {
            return Err(LutParseError::ConflictingDomain);
        }
        if (self.range_1d.is_some() && self.size_1d.is_none())
            || (self.range_3d.is_some() && self.size_3d.is_none())
        {
            return Err(LutParseError::ConflictingDomain);
        }
        let shaper = self.range_1d.map(Domain::uniform).or(shared);
        let cube = self.range_3d.map(Domain::uniform).or(shared);
        let shaper = shaper.unwrap_or(Domain::UNIT);
        let cube = cube.unwrap_or(Domain::UNIT);
        if !shaper.is_valid() || !cube.is_valid() {
            return Err(LutParseError::InvalidDomain);
        }
        Ok((shaper, cube))
    }
}

pub(super) fn parse(source: &[u8]) -> Result<CubeLut, LutParseError> {
    if source.len() > LUT_MAX_SOURCE_BYTES {
        return Err(LutParseError::TooLarge);
    }
    let text = String::from_utf8_lossy(source);
    let mut header = Header::default();
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
                if header.title {
                    return Err(LutParseError::DuplicateDirective("TITLE"));
                }
                header.title = true;
            }
            "LUT_1D_SIZE" => {
                set_once(&mut header.size_1d, "LUT_1D_SIZE", || {
                    parse_size(&mut parts, LUT_1D_MAX_SIZE)
                })?;
            }
            "LUT_3D_SIZE" => {
                set_once(&mut header.size_3d, "LUT_3D_SIZE", || {
                    parse_size(&mut parts, LUT_MAX_SIZE)
                })?;
            }
            "DOMAIN_MIN" => {
                set_once(&mut header.domain_min, "DOMAIN_MIN", || {
                    parse_values(&mut parts)
                })?;
            }
            "DOMAIN_MAX" => {
                set_once(&mut header.domain_max, "DOMAIN_MAX", || {
                    parse_values(&mut parts)
                })?;
            }
            "LUT_1D_INPUT_RANGE" => {
                set_once(&mut header.range_1d, "LUT_1D_INPUT_RANGE", || {
                    parse_values(&mut parts)
                })?;
            }
            "LUT_3D_INPUT_RANGE" => {
                set_once(&mut header.range_3d, "LUT_3D_INPUT_RANGE", || {
                    parse_values(&mut parts)
                })?;
            }
            _ => {
                let r = match parse_number(keyword) {
                    Err(LutParseError::InvalidNumber)
                        if keyword.starts_with(|c: char| c.is_ascii_alphabetic()) =>
                    {
                        return Err(LutParseError::UnknownDirective(keyword.to_string()));
                    }
                    other => other?,
                };
                let [g, b] = parse_values(&mut parts)?;
                data.push([r, g, b]);
            }
        }
    }

    if header.size_1d.is_none() && header.size_3d.is_none() {
        return Err(LutParseError::MissingSize);
    }
    if data.is_empty() {
        return Err(LutParseError::Empty);
    }
    let shaper_len = header.size_1d.unwrap_or(0);
    let cube_len = header.size_3d.map_or(0, |n| n * n * n);
    let expected = shaper_len + cube_len;
    if data.len() != expected {
        return Err(LutParseError::WrongEntryCount {
            expected,
            found: data.len(),
        });
    }
    let (shaper_domain, cube_domain) = header.domains()?;
    let cube_data = data.split_off(shaper_len);
    Ok(CubeLut {
        shaper: header.size_1d.map(|_| Lut1d {
            domain: shaper_domain,
            data,
        }),
        cube: header.size_3d.map(|size| Lut3d {
            size,
            domain: cube_domain,
            data: cube_data,
        }),
    })
}

fn set_once<T>(
    slot: &mut Option<T>,
    name: &'static str,
    value: impl FnOnce() -> Result<T, LutParseError>,
) -> Result<(), LutParseError> {
    if slot.is_some() {
        return Err(LutParseError::DuplicateDirective(name));
    }
    *slot = Some(value()?);
    Ok(())
}

fn parse_size<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    max: usize,
) -> Result<usize, LutParseError> {
    let n: usize = parts
        .next()
        .ok_or(LutParseError::InvalidSize)?
        .parse()
        .map_err(|_| LutParseError::InvalidSize)?;
    if !(LUT_MIN_SIZE..=max).contains(&n) || parts.next().is_some() {
        return Err(LutParseError::InvalidSize);
    }
    Ok(n)
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

fn parse_values<'a, const N: usize>(
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<[f32; N], LutParseError> {
    let mut out = [0.0f32; N];
    for slot in &mut out {
        *slot = parse_number(parts.next().ok_or(LutParseError::InvalidNumber)?)?;
    }
    if parts.next().is_some() {
        return Err(LutParseError::InvalidNumber);
    }
    Ok(out)
}
