mod parse;

#[cfg(test)]
mod tests;

pub use parse::parse_dcp;

use std::sync::Arc;

pub const DCP_MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
const DCP_MAX_TABLE_DIM: u32 = 256;
const DCP_MAX_TABLE_ENTRIES: usize = DCP_MAX_SOURCE_BYTES / (3 * size_of::<f32>());

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
    fn expected_len(&self) -> Option<usize> {
        (self.hue_div as usize)
            .checked_mul(self.sat_div as usize)?
            .checked_mul(self.val_div.max(1) as usize)
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
    pub huesatmap1: Option<Arc<HueSatMap>>,
    pub huesatmap2: Option<Arc<HueSatMap>>,
    pub look_table: Option<Arc<HueSatMap>>,
    pub tone_curve: Option<Arc<Vec<[f32; 2]>>>,
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

    pub fn has_forward_matrix(&self) -> bool {
        self.forward_matrix1.is_some()
    }

    pub fn is_adobe(&self) -> bool {
        self.copyright
            .as_deref()
            .is_some_and(|c| c.contains("Adobe"))
    }
}
