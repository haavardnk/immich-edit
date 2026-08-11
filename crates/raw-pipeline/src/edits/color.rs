use serde::{Deserialize, Serialize};

pub const HSL_BANDS: usize = 8;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct HslBand {
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub sat: f64,
    #[serde(default)]
    pub lum: f64,
}

impl HslBand {
    pub fn is_zero(&self) -> bool {
        self.hue == 0.0 && self.sat == 0.0 && self.lum == 0.0
    }

    pub fn clamped(&self) -> Self {
        Self {
            hue: self.hue.clamp(-100.0, 100.0),
            sat: self.sat.clamp(-100.0, 100.0),
            lum: self.lum.clamp(-100.0, 100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HslEdits {
    #[serde(default = "default_bands")]
    pub bands: [HslBand; HSL_BANDS],
}

fn default_bands() -> [HslBand; HSL_BANDS] {
    [HslBand::default(); HSL_BANDS]
}

impl Default for HslEdits {
    fn default() -> Self {
        Self {
            bands: default_bands(),
        }
    }
}

impl HslEdits {
    pub fn is_zero(&self) -> bool {
        self.bands.iter().all(|b| b.is_zero())
    }

    pub fn clamped(&self) -> Self {
        let mut bands = [HslBand::default(); HSL_BANDS];
        for (i, b) in bands.iter_mut().enumerate() {
            *b = self.bands[i].clamped();
        }
        Self { bands }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorGradeRegion {
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub sat: f64,
    #[serde(default)]
    pub lum: f64,
}

impl ColorGradeRegion {
    pub fn is_zero(&self) -> bool {
        self.sat == 0.0 && self.lum == 0.0
    }

    pub fn clamped(&self) -> Self {
        Self {
            hue: self.hue.rem_euclid(360.0),
            sat: self.sat.clamp(0.0, 100.0),
            lum: self.lum.clamp(-50.0, 50.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorGradeEdits {
    #[serde(default)]
    pub shadows: ColorGradeRegion,
    #[serde(default)]
    pub midtones: ColorGradeRegion,
    #[serde(default)]
    pub highlights: ColorGradeRegion,
    #[serde(default)]
    pub global: ColorGradeRegion,
    #[serde(default)]
    pub balance: f64,
    #[serde(default)]
    pub blend: f64,
}

impl ColorGradeEdits {
    pub fn is_zero(&self) -> bool {
        self.shadows.is_zero()
            && self.midtones.is_zero()
            && self.highlights.is_zero()
            && self.global.is_zero()
    }

    pub fn clamped(&self) -> Self {
        Self {
            shadows: self.shadows.clamped(),
            midtones: self.midtones.clamped(),
            highlights: self.highlights.clamped(),
            global: self.global.clamped(),
            balance: self.balance.clamp(-100.0, 100.0),
            blend: self.blend.clamp(0.0, 100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lut3dEdits {
    #[serde(default)]
    pub lut_id: Option<String>,
    #[serde(default = "lut_amount_default")]
    pub amount: f64,
}

fn lut_amount_default() -> f64 {
    100.0
}

impl Default for Lut3dEdits {
    fn default() -> Self {
        Self {
            lut_id: None,
            amount: lut_amount_default(),
        }
    }
}

impl Lut3dEdits {
    pub fn is_active(&self) -> bool {
        self.lut_id.as_ref().is_some_and(|id| !id.is_empty()) && self.amount > 0.0
    }

    pub fn clamped(&self) -> Self {
        Self {
            lut_id: self.lut_id.as_ref().filter(|id| !id.is_empty()).cloned(),
            amount: self.amount.clamp(0.0, 100.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DcpMode {
    Off,
    #[default]
    Auto,
    Profile,
    Flat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DcpEdits {
    #[serde(default)]
    pub mode: DcpMode,
    #[serde(default)]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub illuminant: crate::dcp::DcpIlluminant,
    #[serde(default = "bool_true")]
    pub use_tone_curve: bool,
    #[serde(default = "bool_true")]
    pub use_base_table: bool,
    #[serde(default = "bool_true")]
    pub use_look_table: bool,
    #[serde(default = "bool_true")]
    pub use_baseline_exposure: bool,
}

fn bool_true() -> bool {
    true
}

impl Default for DcpEdits {
    fn default() -> Self {
        Self {
            mode: DcpMode::Auto,
            profile_id: None,
            illuminant: crate::dcp::DcpIlluminant::default(),
            use_tone_curve: true,
            use_base_table: true,
            use_look_table: true,
            use_baseline_exposure: true,
        }
    }
}

impl DcpEdits {
    pub fn is_active(&self) -> bool {
        match self.mode {
            DcpMode::Off | DcpMode::Flat => false,
            DcpMode::Auto => true,
            DcpMode::Profile => self.profile_id.as_ref().is_some_and(|s| !s.is_empty()),
        }
    }

    pub fn is_flat(&self) -> bool {
        self.mode == DcpMode::Flat
    }

    pub fn clamped(&self) -> Self {
        let profile_id = self.profile_id.as_ref().filter(|s| !s.is_empty()).cloned();
        let mode = if self.mode == DcpMode::Profile && profile_id.is_none() {
            DcpMode::Off
        } else {
            self.mode
        };
        Self {
            mode,
            profile_id,
            illuminant: self.illuminant,
            use_tone_curve: self.use_tone_curve,
            use_base_table: self.use_base_table,
            use_look_table: self.use_look_table,
            use_baseline_exposure: self.use_baseline_exposure,
        }
    }

    pub fn referenced_profile_id(&self) -> Option<String> {
        if self.mode == DcpMode::Profile {
            self.profile_id.as_ref().filter(|s| !s.is_empty()).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorEdits {
    #[serde(default)]
    pub hsl: HslEdits,
    #[serde(default)]
    pub color_grade: ColorGradeEdits,
    #[serde(default)]
    pub lut_3d: Lut3dEdits,
    #[serde(default)]
    pub dcp: DcpEdits,
}

impl ColorEdits {
    pub fn clamped(&self) -> Self {
        Self {
            hsl: self.hsl.clamped(),
            color_grade: self.color_grade.clamped(),
            lut_3d: self.lut_3d.clamped(),
            dcp: self.dcp.clamped(),
        }
    }
}
