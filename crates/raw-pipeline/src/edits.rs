mod color;
mod curves;
mod geometry;
mod lens;
mod masks;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use color::{
    ColorEdits, ColorGradeEdits, ColorGradeRegion, DcpEdits, DcpMode, HSL_BANDS, HslBand, HslEdits,
    Lut3dEdits,
};
pub use curves::{CURVE_LUT_SIZE, CurvePoint, CurvePoints, CurvesEdits};
pub use geometry::{AspectLock, CropRect, GeometryEdits};
pub use lens::LensEdits;
pub use masks::{
    ClickPointMeta, GeneratedMeta, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer,
    MaskSource, MaskedEdits, N_MAX_COMPONENTS_PER_LAYER, N_MAX_MASK_LAYERS, N_MAX_POLYGON_POINTS,
    N_MAX_RASTER_SLOTS, N_MAX_TOTAL_COMPONENTS, RangeMeta, Vec2f,
};

use masks::clamp_masks;

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

impl BasicEdits {
    pub fn clamped(&self) -> Self {
        Self {
            exposure_ev: self.exposure_ev.clamp(-5.0, 5.0),
            brightness: self.brightness.clamp(-100.0, 100.0),
            contrast: self.contrast.clamp(-100.0, 100.0),
            saturation: self.saturation.clamp(-100.0, 100.0),
            vibrance: self.vibrance.clamp(-100.0, 100.0),
            wb_temp: self.wb_temp.clamp(-100.0, 100.0),
            wb_tint: self.wb_tint.clamp(-100.0, 100.0),
            texture: self.texture.clamp(-100.0, 100.0),
            clarity: self.clarity.clamp(-100.0, 100.0),
            dehaze: self.dehaze.clamp(-100.0, 100.0),
            curves: self.curves.clone(),
        }
    }
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

impl ToneEdits {
    pub fn clamped(&self) -> Self {
        Self {
            highlights: self.highlights.clamp(-100.0, 100.0),
            shadows: self.shadows.clamp(-100.0, 100.0),
            blacks: self.blacks.clamp(-100.0, 100.0),
            whites: self.whites.clamp(-100.0, 100.0),
        }
    }
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
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub unknown_ops: std::collections::BTreeMap<String, serde_json::Value>,
}

impl Edits {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    pub fn masked_sharpen_active(&self) -> bool {
        self.masks
            .iter()
            .any(|l| l.is_effective() && l.edits.sharpen.is_some())
    }

    pub fn clamped(&self) -> Self {
        Self {
            basic: self.basic.clamped(),
            tone: self.tone.clamped(),
            color: self.color.clamped(),
            detail: self.detail.clamped(),
            effects: self.effects.clamped(),
            lens: self.lens.clamped(),
            geometry: self.geometry.clamped(),
            masks: clamp_masks(&self.masks),
            unknown_ops: self.unknown_ops.clone(),
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

    pub fn retained_raster_ids(&self) -> Vec<String> {
        let mut out = self.referenced_raster_ids();
        for layer in &self.masks {
            for comp in &layer.components {
                let Some(meta) = &comp.generated else {
                    continue;
                };
                if !meta.prob_raster_id.is_empty() && !out.iter().any(|s| s == &meta.prob_raster_id)
                {
                    out.push(meta.prob_raster_id.clone());
                }
            }
        }
        out
    }

    pub fn referenced_lut_id(&self) -> Option<String> {
        self.color
            .lut_3d
            .lut_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .cloned()
    }
}
