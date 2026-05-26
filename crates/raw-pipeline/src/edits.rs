use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CURVE_LUT_SIZE: usize = 16;

pub const N_MAX_MASK_LAYERS: usize = 8;
pub const N_MAX_COMPONENTS_PER_LAYER: usize = 8;
pub const N_MAX_TOTAL_COMPONENTS: usize = 32;
pub const N_MAX_RASTER_SLOTS: usize = 16;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct CurvePoints {
    pub points: Vec<CurvePoint>,
}

impl<'de> Deserialize<'de> for CurvePoints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use serde_json::Value;

        let v = Value::deserialize(deserializer)?;
        let Value::Array(arr) = v else {
            return Err(de::Error::custom("expected array for curves"));
        };
        let pts: Vec<CurvePoint> = arr
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect();
        if pts.len() >= 2 {
            Ok(Self { points: pts })
        } else {
            Ok(Self::default())
        }
    }
}

impl CurvePoints {
    fn default_points() -> Vec<CurvePoint> {
        vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 1.0, y: 1.0 }]
    }

    pub fn is_identity(&self) -> bool {
        self.points.len() == 2
            && self.points[0].x.abs() < 1e-10
            && self.points[0].y.abs() < 1e-10
            && (self.points[1].x - 1.0).abs() < 1e-10
            && (self.points[1].y - 1.0).abs() < 1e-10
    }

    pub fn as_tuples(&self) -> Vec<(f64, f64)> {
        self.points.iter().map(|p| (p.x, p.y)).collect()
    }
}

impl Default for CurvePoints {
    fn default() -> Self {
        Self {
            points: Self::default_points(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CurvesEdits {
    #[serde(default)]
    pub composite: CurvePoints,
    #[serde(default)]
    pub r: CurvePoints,
    #[serde(default)]
    pub g: CurvePoints,
    #[serde(default)]
    pub b: CurvePoints,
    #[serde(default)]
    pub luma: CurvePoints,
}

impl CurvesEdits {
    pub fn is_identity(&self) -> bool {
        self.composite.is_identity()
            && self.r.is_identity()
            && self.g.is_identity()
            && self.b.is_identity()
            && self.luma.is_identity()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BasicEdits {
    #[serde(default)]
    pub exposure_ev: f64,
    #[serde(default)]
    pub brightness: f64,
    #[serde(default)]
    pub contrast: f64,
    #[serde(default)]
    pub saturation: f64,
    #[serde(default)]
    pub vibrance: f64,
    #[serde(default)]
    pub wb_temp: f64,
    #[serde(default)]
    pub wb_tint: f64,
    #[serde(default)]
    pub texture: f64,
    #[serde(default)]
    pub clarity: f64,
    #[serde(default)]
    pub dehaze: f64,
    #[serde(default)]
    pub curves: CurvesEdits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToneEdits {
    #[serde(default)]
    pub highlights: f64,
    #[serde(default)]
    pub shadows: f64,
    #[serde(default)]
    pub blacks: f64,
    #[serde(default)]
    pub whites: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorEdits {
    #[serde(default)]
    pub hsl: HslEdits,
    #[serde(default)]
    pub color_grade: ColorGradeEdits,
}

fn sharpen_radius_default() -> f64 {
    1.0
}
fn sharpen_detail_default() -> f64 {
    25.0
}
fn luma_nr_detail_default() -> f64 {
    50.0
}
fn color_nr_detail_default() -> f64 {
    50.0
}
fn color_nr_smoothness_default() -> f64 {
    50.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DetailEdits {
    #[serde(default)]
    pub sharpen_amount: f64,
    #[serde(default = "sharpen_radius_default")]
    pub sharpen_radius: f64,
    #[serde(default = "sharpen_detail_default")]
    pub sharpen_detail: f64,
    #[serde(default)]
    pub sharpen_masking: f64,
    #[serde(default)]
    pub luma_nr_amount: f64,
    #[serde(default = "luma_nr_detail_default")]
    pub luma_nr_detail: f64,
    #[serde(default)]
    pub luma_nr_contrast: f64,
    #[serde(default)]
    pub color_nr_amount: f64,
    #[serde(default = "color_nr_detail_default")]
    pub color_nr_detail: f64,
    #[serde(default = "color_nr_smoothness_default")]
    pub color_nr_smoothness: f64,
}

impl Default for DetailEdits {
    fn default() -> Self {
        Self {
            sharpen_amount: 0.0,
            sharpen_radius: sharpen_radius_default(),
            sharpen_detail: sharpen_detail_default(),
            sharpen_masking: 0.0,
            luma_nr_amount: 0.0,
            luma_nr_detail: luma_nr_detail_default(),
            luma_nr_contrast: 0.0,
            color_nr_amount: 0.0,
            color_nr_detail: color_nr_detail_default(),
            color_nr_smoothness: color_nr_smoothness_default(),
        }
    }
}

impl DetailEdits {
    pub fn sharpen_active(&self) -> bool {
        self.sharpen_amount != 0.0
    }
    pub fn luma_nr_active(&self) -> bool {
        self.luma_nr_amount != 0.0
    }
    pub fn color_nr_active(&self) -> bool {
        self.color_nr_amount != 0.0
    }
    pub fn any_active(&self) -> bool {
        self.sharpen_active() || self.luma_nr_active() || self.color_nr_active()
    }
    pub fn clamped(&self) -> Self {
        let mut out = Self {
            sharpen_amount: self.sharpen_amount.clamp(0.0, 150.0),
            sharpen_radius: self.sharpen_radius.clamp(0.5, 3.0),
            sharpen_detail: self.sharpen_detail.clamp(0.0, 100.0),
            sharpen_masking: self.sharpen_masking.clamp(0.0, 100.0),
            luma_nr_amount: self.luma_nr_amount.clamp(0.0, 100.0),
            luma_nr_detail: self.luma_nr_detail.clamp(0.0, 100.0),
            luma_nr_contrast: self.luma_nr_contrast.clamp(0.0, 100.0),
            color_nr_amount: self.color_nr_amount.clamp(0.0, 100.0),
            color_nr_detail: self.color_nr_detail.clamp(0.0, 100.0),
            color_nr_smoothness: self.color_nr_smoothness.clamp(0.0, 100.0),
        };
        if !out.sharpen_active() {
            out.sharpen_radius = sharpen_radius_default();
            out.sharpen_detail = sharpen_detail_default();
            out.sharpen_masking = 0.0;
        }
        if !out.luma_nr_active() {
            out.luma_nr_detail = luma_nr_detail_default();
            out.luma_nr_contrast = 0.0;
        }
        if !out.color_nr_active() {
            out.color_nr_detail = color_nr_detail_default();
            out.color_nr_smoothness = color_nr_smoothness_default();
        }
        out
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CropRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl CropRect {
    pub fn full() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }

    pub fn is_full(&self) -> bool {
        (self.x).abs() < 1e-5
            && (self.y).abs() < 1e-5
            && (self.w - 1.0).abs() < 1e-5
            && (self.h - 1.0).abs() < 1e-5
    }

    pub fn clamped(&self) -> Self {
        let x = self.x.clamp(0.0, 1.0);
        let y = self.y.clamp(0.0, 1.0);
        let w = self.w.clamp(0.0, 1.0 - x);
        let h = self.h.clamp(0.0, 1.0 - y);
        Self { x, y, w, h }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AspectLock {
    #[default]
    Original,
    Free,
    Ratio {
        num: u16,
        den: u16,
    },
}

fn vignette_midpoint_default() -> f64 {
    50.0
}
fn vignette_feather_default() -> f64 {
    50.0
}
fn grain_size_default() -> f64 {
    25.0
}
fn grain_roughness_default() -> f64 {
    50.0
}

fn lens_amount_default() -> f64 {
    100.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LensEdits {
    #[serde(default)]
    pub profile_enabled: bool,
    #[serde(default)]
    pub ca_enabled: bool,
    #[serde(default)]
    pub constrain_crop: bool,
    #[serde(default = "lens_amount_default")]
    pub distortion_amount: f64,
    #[serde(default = "lens_amount_default")]
    pub vignette_amount: f64,
    #[serde(default)]
    pub k1: f64,
    #[serde(default)]
    pub k2: f64,
    #[serde(default)]
    pub k3: f64,
    #[serde(default)]
    pub vk1: f64,
    #[serde(default)]
    pub vk2: f64,
    #[serde(default)]
    pub vk3: f64,
    #[serde(default)]
    pub ca_red_scale_x10000: f64,
    #[serde(default)]
    pub ca_blue_scale_x10000: f64,
}

impl Default for LensEdits {
    fn default() -> Self {
        Self {
            profile_enabled: false,
            ca_enabled: false,
            constrain_crop: false,
            distortion_amount: 100.0,
            vignette_amount: 100.0,
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            vk1: 0.0,
            vk2: 0.0,
            vk3: 0.0,
            ca_red_scale_x10000: 0.0,
            ca_blue_scale_x10000: 0.0,
        }
    }
}

impl LensEdits {
    pub fn distortion_active(&self) -> bool {
        self.profile_enabled
            && self.distortion_amount != 0.0
            && (self.k1 != 0.0 || self.k2 != 0.0 || self.k3 != 0.0)
    }
    pub fn vignette_active(&self) -> bool {
        self.profile_enabled
            && self.vignette_amount != 0.0
            && (self.vk1 != 0.0 || self.vk2 != 0.0 || self.vk3 != 0.0)
    }
    pub fn ca_active(&self) -> bool {
        self.ca_enabled && (self.ca_red_scale_x10000 != 0.0 || self.ca_blue_scale_x10000 != 0.0)
    }
    pub fn any_active(&self) -> bool {
        self.distortion_active() || self.vignette_active() || self.ca_active()
    }
    pub fn effective_k(&self) -> (f64, f64, f64) {
        let s = self.distortion_amount / 100.0;
        (self.k1 * s, self.k2 * s, self.k3 * s)
    }
    pub fn effective_vk(&self) -> (f64, f64, f64) {
        let s = self.vignette_amount / 100.0;
        (self.vk1 * s, self.vk2 * s, self.vk3 * s)
    }
    pub fn ca_scales(&self) -> (f64, f64) {
        (
            1.0 + self.ca_red_scale_x10000 / 10000.0,
            1.0 + self.ca_blue_scale_x10000 / 10000.0,
        )
    }
    pub fn clamped(&self) -> Self {
        Self {
            profile_enabled: self.profile_enabled,
            ca_enabled: self.ca_enabled,
            constrain_crop: self.constrain_crop,
            distortion_amount: self.distortion_amount.clamp(0.0, 200.0),
            vignette_amount: self.vignette_amount.clamp(0.0, 200.0),
            k1: self.k1,
            k2: self.k2,
            k3: self.k3,
            vk1: self.vk1,
            vk2: self.vk2,
            vk3: self.vk3,
            ca_red_scale_x10000: self.ca_red_scale_x10000.clamp(-500.0, 500.0),
            ca_blue_scale_x10000: self.ca_blue_scale_x10000.clamp(-500.0, 500.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EffectsEdits {
    #[serde(default)]
    pub vignette_amount: f64,
    #[serde(default = "vignette_midpoint_default")]
    pub vignette_midpoint: f64,
    #[serde(default = "vignette_feather_default")]
    pub vignette_feather: f64,
    #[serde(default)]
    pub vignette_roundness: f64,
    #[serde(default)]
    pub grain_amount: f64,
    #[serde(default = "grain_size_default")]
    pub grain_size: f64,
    #[serde(default = "grain_roughness_default")]
    pub grain_roughness: f64,
}

impl Default for EffectsEdits {
    fn default() -> Self {
        Self {
            vignette_amount: 0.0,
            vignette_midpoint: vignette_midpoint_default(),
            vignette_feather: vignette_feather_default(),
            vignette_roundness: 0.0,
            grain_amount: 0.0,
            grain_size: grain_size_default(),
            grain_roughness: grain_roughness_default(),
        }
    }
}

impl EffectsEdits {
    pub fn vignette_active(&self) -> bool {
        self.vignette_amount != 0.0
    }
    pub fn grain_active(&self) -> bool {
        self.grain_amount != 0.0
    }
    pub fn any_active(&self) -> bool {
        self.vignette_active() || self.grain_active()
    }
    pub fn clamped(&self) -> Self {
        let mut out = Self {
            vignette_amount: self.vignette_amount.clamp(-100.0, 100.0),
            vignette_midpoint: self.vignette_midpoint.clamp(0.0, 100.0),
            vignette_feather: self.vignette_feather.clamp(0.0, 100.0),
            vignette_roundness: self.vignette_roundness.clamp(-100.0, 100.0),
            grain_amount: self.grain_amount.clamp(0.0, 100.0),
            grain_size: self.grain_size.clamp(0.0, 100.0),
            grain_roughness: self.grain_roughness.clamp(0.0, 100.0),
        };
        if !out.vignette_active() {
            out.vignette_midpoint = vignette_midpoint_default();
            out.vignette_feather = vignette_feather_default();
            out.vignette_roundness = 0.0;
        }
        if !out.grain_active() {
            out.grain_size = grain_size_default();
            out.grain_roughness = grain_roughness_default();
        }
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GeometryEdits {
    #[serde(default)]
    pub rotate: u16,
    #[serde(default)]
    pub rotate_angle: f32,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
    #[serde(default)]
    pub crop: Option<CropRect>,
    #[serde(default)]
    pub aspect: AspectLock,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaskComponentMode {
    #[default]
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaskSource {
    #[default]
    Manual,
    Generated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaskComponentKind {
    Linear {
        p0: Vec2f,
        p1: Vec2f,
        #[serde(default)]
        feather: f32,
    },
    Radial {
        center: Vec2f,
        radius_xy: Vec2f,
        #[serde(default)]
        feather: f32,
    },
    Brush {
        raster_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MaskedEdits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure_ev: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saturation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vibrance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wb_temp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wb_tint: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlights: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadows: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whites: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blacks: Option<f64>,
}

impl MaskedEdits {
    pub fn is_zero(&self) -> bool {
        self.exposure_ev.is_none()
            && self.brightness.is_none()
            && self.contrast.is_none()
            && self.saturation.is_none()
            && self.vibrance.is_none()
            && self.wb_temp.is_none()
            && self.wb_tint.is_none()
            && self.highlights.is_none()
            && self.shadows.is_none()
            && self.whites.is_none()
            && self.blacks.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskComponent {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub mode: MaskComponentMode,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub invert: bool,
    pub kind: MaskComponentKind,
    #[serde(default)]
    pub source: MaskSource,
}

fn default_true() -> bool {
    true
}
fn default_opacity() -> f32 {
    1.0
}
fn default_color() -> String {
    "#ff3b30".into()
}
fn default_amount() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskLayer {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_amount")]
    pub amount: f32,
    #[serde(default)]
    pub components: Vec<MaskComponent>,
    #[serde(default)]
    pub edits: MaskedEdits,
}

impl MaskLayer {
    pub fn is_effective(&self) -> bool {
        if !self.enabled || self.amount.abs() < 1e-6 {
            return false;
        }
        let has_component = self
            .components
            .iter()
            .any(|c| c.enabled && c.opacity.abs() > 1e-6);
        has_component && !self.edits.is_zero()
    }
}

fn clamp_masked_delta(v: Option<f64>, lo: f64, hi: f64) -> Option<f64> {
    let val = v?;
    if val == 0.0 {
        return None;
    }
    Some(val.clamp(lo, hi))
}

fn clamp_masked_edits(m: &MaskedEdits) -> MaskedEdits {
    MaskedEdits {
        exposure_ev: clamp_masked_delta(m.exposure_ev, -5.0, 5.0),
        brightness: clamp_masked_delta(m.brightness, -100.0, 100.0),
        contrast: clamp_masked_delta(m.contrast, -100.0, 100.0),
        saturation: clamp_masked_delta(m.saturation, -100.0, 100.0),
        vibrance: clamp_masked_delta(m.vibrance, -100.0, 100.0),
        wb_temp: clamp_masked_delta(m.wb_temp, -100.0, 100.0),
        wb_tint: clamp_masked_delta(m.wb_tint, -100.0, 100.0),
        highlights: clamp_masked_delta(m.highlights, -100.0, 100.0),
        shadows: clamp_masked_delta(m.shadows, -100.0, 100.0),
        whites: clamp_masked_delta(m.whites, -100.0, 100.0),
        blacks: clamp_masked_delta(m.blacks, -100.0, 100.0),
    }
}

fn clamp_component(c: &MaskComponent) -> MaskComponent {
    let kind = match &c.kind {
        MaskComponentKind::Linear { p0, p1, feather } => MaskComponentKind::Linear {
            p0: *p0,
            p1: *p1,
            feather: feather.clamp(0.0, 1.0),
        },
        MaskComponentKind::Radial {
            center,
            radius_xy,
            feather,
        } => MaskComponentKind::Radial {
            center: *center,
            radius_xy: Vec2f {
                x: radius_xy.x.clamp(0.0, 2.0),
                y: radius_xy.y.clamp(0.0, 2.0),
            },
            feather: feather.clamp(0.0, 1.0),
        },
        MaskComponentKind::Brush { raster_id } => MaskComponentKind::Brush {
            raster_id: raster_id.clone(),
        },
    };
    MaskComponent {
        id: c.id.clone(),
        enabled: c.enabled,
        mode: c.mode,
        opacity: c.opacity.clamp(0.0, 1.0),
        invert: c.invert,
        kind,
        source: c.source,
    }
}

fn clamp_masks(layers: &[MaskLayer]) -> Vec<MaskLayer> {
    let mut total_components: usize = 0;
    let mut out: Vec<MaskLayer> = Vec::new();
    for layer in layers.iter().take(N_MAX_MASK_LAYERS) {
        let remaining = N_MAX_TOTAL_COMPONENTS.saturating_sub(total_components);
        let take = remaining.min(N_MAX_COMPONENTS_PER_LAYER);
        let components: Vec<MaskComponent> = layer
            .components
            .iter()
            .take(take)
            .map(clamp_component)
            .collect();
        total_components += components.len();
        out.push(MaskLayer {
            id: layer.id.clone(),
            name: layer.name.clone(),
            enabled: layer.enabled,
            color: layer.color.clone(),
            amount: layer.amount.clamp(0.0, 1.0),
            components,
            edits: clamp_masked_edits(&layer.edits),
        });
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Edits {
    #[serde(default)]
    pub basic: BasicEdits,
    #[serde(default)]
    pub tone: ToneEdits,
    #[serde(default)]
    pub color: ColorEdits,
    #[serde(default)]
    pub detail: DetailEdits,
    #[serde(default)]
    pub effects: EffectsEdits,
    #[serde(default)]
    pub lens: LensEdits,
    #[serde(default)]
    pub geometry: GeometryEdits,
    #[serde(default)]
    pub masks: Vec<MaskLayer>,
}

impl Edits {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    pub fn clamped(&self) -> Self {
        let rotate = match self.geometry.rotate {
            0 | 90 | 180 | 270 => self.geometry.rotate,
            _ => 0,
        };
        Self {
            basic: BasicEdits {
                exposure_ev: self.basic.exposure_ev.clamp(-5.0, 5.0),
                brightness: self.basic.brightness.clamp(-100.0, 100.0),
                contrast: self.basic.contrast.clamp(-100.0, 100.0),
                saturation: self.basic.saturation.clamp(-100.0, 100.0),
                vibrance: self.basic.vibrance.clamp(-100.0, 100.0),
                wb_temp: self.basic.wb_temp.clamp(-100.0, 100.0),
                wb_tint: self.basic.wb_tint.clamp(-100.0, 100.0),
                texture: self.basic.texture.clamp(-100.0, 100.0),
                clarity: self.basic.clarity.clamp(-100.0, 100.0),
                dehaze: self.basic.dehaze.clamp(-100.0, 100.0),
                curves: self.basic.curves.clone(),
            },
            tone: ToneEdits {
                highlights: self.tone.highlights.clamp(-100.0, 100.0),
                shadows: self.tone.shadows.clamp(-100.0, 100.0),
                blacks: self.tone.blacks.clamp(-100.0, 100.0),
                whites: self.tone.whites.clamp(-100.0, 100.0),
            },
            color: ColorEdits {
                hsl: self.color.hsl.clamped(),
                color_grade: self.color.color_grade.clamped(),
            },
            detail: self.detail.clamped(),
            effects: self.effects.clamped(),
            lens: self.lens.clamped(),
            geometry: GeometryEdits {
                rotate,
                rotate_angle: self.geometry.rotate_angle.clamp(-45.0, 45.0),
                flip_h: self.geometry.flip_h,
                flip_v: self.geometry.flip_v,
                crop: self.geometry.crop.map(|c| c.clamped()),
                aspect: self.geometry.aspect,
            },
            masks: clamp_masks(&self.masks),
        }
    }

    pub fn stable_hash(&self) -> String {
        let clamped = self.clamped();
        let json = serde_json::to_string(&clamped).expect("edits serialize");
        let hash = Sha256::digest(json.as_bytes());
        hex::encode(&hash[..16])
    }

    pub fn referenced_raster_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for layer in &self.masks {
            for comp in &layer.components {
                if let MaskComponentKind::Brush { raster_id } = &comp.kind {
                    if !raster_id.is_empty() && !out.iter().any(|s| s == raster_id) {
                        out.push(raster_id.clone());
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        let e = Edits::default();
        assert!(e.is_identity());
    }

    #[test]
    fn clamp_exposure() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 10.0;
        let c = e.clamped();
        assert_eq!(c.basic.exposure_ev, 5.0);
    }

    #[test]
    fn clamp_invalid_rotate() {
        let mut e = Edits::default();
        e.geometry.rotate = 45;
        let c = e.clamped();
        assert_eq!(c.geometry.rotate, 0);
    }

    #[test]
    fn stable_hash_deterministic() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 1.5;
        let h1 = e.stable_hash();
        let h2 = e.stable_hash();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn stable_hash_differs_on_change() {
        let mut a = Edits::default();
        a.basic.exposure_ev = 1.0;
        let mut b = Edits::default();
        b.basic.exposure_ev = 2.0;
        assert_ne!(a.stable_hash(), b.stable_hash());
    }

    #[test]
    fn serde_roundtrip() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 1.0;
        e.geometry.rotate = 90;
        let json = serde_json::to_string(&e).unwrap();
        let e2: Edits = serde_json::from_str(&json).unwrap();
        assert_eq!(e, e2);
    }

    #[test]
    fn serde_defaults() {
        let json = "{}";
        let e: Edits = serde_json::from_str(json).unwrap();
        assert!(e.is_identity());
    }

    #[test]
    fn mask_brush_serde_roundtrip_preserves_raster_id() {
        let mut e = Edits::default();
        e.masks.push(MaskLayer {
            id: "l1".into(),
            name: "brush layer".into(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            components: vec![MaskComponent {
                id: "c1".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                opacity: 1.0,
                invert: false,
                kind: MaskComponentKind::Brush {
                    raster_id: "abc123".into(),
                },
                source: MaskSource::Manual,
            }],
            edits: MaskedEdits {
                exposure_ev: Some(1.0),
                ..Default::default()
            },
        });
        let json = serde_json::to_string(&e).unwrap();
        let e2: Edits = serde_json::from_str(&json).unwrap();
        assert_eq!(e, e2);
        match &e2.masks[0].components[0].kind {
            MaskComponentKind::Brush { raster_id } => assert_eq!(raster_id, "abc123"),
            _ => panic!("expected brush kind"),
        }
    }

    #[test]
    fn referenced_raster_ids_dedups() {
        let mut e = Edits::default();
        let make_comp = |id: &str, raster: &str| MaskComponent {
            id: id.into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            opacity: 1.0,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: raster.into(),
            },
            source: MaskSource::Manual,
        };
        e.masks.push(MaskLayer {
            id: "l1".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![make_comp("a", "r1"), make_comp("b", "r2")],
            edits: MaskedEdits::default(),
        });
        e.masks.push(MaskLayer {
            id: "l2".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![make_comp("c", "r1")],
            edits: MaskedEdits::default(),
        });
        let ids = e.referenced_raster_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"r1".to_string()));
        assert!(ids.contains(&"r2".to_string()));
    }

    #[test]
    fn clamped_preserves_brush_raster_id() {
        let mut e = Edits::default();
        e.masks.push(MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 2.0,
            components: vec![MaskComponent {
                id: "c".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                opacity: 2.0,
                invert: false,
                kind: MaskComponentKind::Brush {
                    raster_id: "keep-me".into(),
                },
                source: MaskSource::Manual,
            }],
            edits: MaskedEdits::default(),
        });
        let c = e.clamped();
        assert_eq!(c.masks[0].amount, 1.0);
        assert_eq!(c.masks[0].components[0].opacity, 1.0);
        match &c.masks[0].components[0].kind {
            MaskComponentKind::Brush { raster_id } => assert_eq!(raster_id, "keep-me"),
            _ => panic!("expected brush kind"),
        }
    }
}
