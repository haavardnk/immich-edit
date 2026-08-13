use serde::{Deserialize, Serialize};

pub const N_MAX_MASK_LAYERS: usize = 8;
pub const N_MAX_COMPONENTS_PER_LAYER: usize = 8;
pub const N_MAX_TOTAL_COMPONENTS: usize = 32;
pub const N_MAX_RASTER_SLOTS: usize = 16;
pub const N_MAX_POLYGON_POINTS: usize = 64;
pub const N_MAX_CLICK_POINTS: usize = 32;
pub const MAX_REFINE_PX: f32 = 128.0;

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
    LumaRange {
        min: f32,
        max: f32,
        softness: f32,
    },
    ColorRange {
        sample_rgb: [f32; 3],
        tolerance: f32,
        softness: f32,
    },
    Polygon {
        points: Vec<Vec2f>,
        #[serde(default)]
        feather: f32,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sharpen: Option<f64>,
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
            && self.texture.is_none()
            && self.clarity.is_none()
            && self.sharpen.is_none()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ClickPointMeta {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_true")]
    pub positive: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RangeMeta {
    pub min: f32,
    pub max: f32,
    #[serde(default)]
    pub softness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedMeta {
    pub model_id: String,
    pub kind: String,
    pub prob_raster_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub painted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<ClickPointMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskComponent {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub mode: MaskComponentMode,
    #[serde(default)]
    pub invert: bool,
    pub kind: MaskComponentKind,
    #[serde(default)]
    pub source: MaskSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated: Option<GeneratedMeta>,
}

fn default_true() -> bool {
    true
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
    pub invert: bool,
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
        let has_component = self.components.iter().any(|c| c.enabled);
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
        texture: clamp_masked_delta(m.texture, -100.0, 100.0),
        clarity: clamp_masked_delta(m.clarity, -100.0, 100.0),
        sharpen: clamp_masked_delta(m.sharpen, -150.0, 150.0),
    }
}

fn clamp_generated(meta: &GeneratedMeta) -> GeneratedMeta {
    GeneratedMeta {
        model_id: meta.model_id.clone(),
        kind: meta.kind.clone(),
        prob_raster_id: meta.prob_raster_id.clone(),
        class: meta.class.clone(),
        grow: meta.grow.clamp(-MAX_REFINE_PX, MAX_REFINE_PX),
        feather: meta.feather.clamp(0.0, MAX_REFINE_PX),
        painted: meta.painted,
        points: meta
            .points
            .iter()
            .take(N_MAX_CLICK_POINTS)
            .map(|p| ClickPointMeta {
                x: p.x.clamp(0.0, 1.0),
                y: p.y.clamp(0.0, 1.0),
                positive: p.positive,
            })
            .collect(),
        range: meta.range.map(|r| RangeMeta {
            min: r.min.min(r.max).clamp(0.0, 1.0),
            max: r.min.max(r.max).clamp(0.0, 1.0),
            softness: r.softness.clamp(0.0, 1.0),
        }),
    }
}

pub(super) fn clamp_point(p: Vec2f) -> Vec2f {
    Vec2f {
        x: p.x.clamp(-1.0, 2.0),
        y: p.y.clamp(-1.0, 2.0),
    }
}

fn clamp_component(c: &MaskComponent) -> MaskComponent {
    let kind = match &c.kind {
        MaskComponentKind::Linear { p0, p1, feather } => MaskComponentKind::Linear {
            p0: clamp_point(*p0),
            p1: clamp_point(*p1),
            feather: feather.clamp(0.0, 1.0),
        },
        MaskComponentKind::Radial {
            center,
            radius_xy,
            feather,
        } => MaskComponentKind::Radial {
            center: clamp_point(*center),
            radius_xy: Vec2f {
                x: radius_xy.x.clamp(0.0, 2.0),
                y: radius_xy.y.clamp(0.0, 2.0),
            },
            feather: feather.clamp(0.0, 1.0),
        },
        MaskComponentKind::Brush { raster_id } => MaskComponentKind::Brush {
            raster_id: raster_id.clone(),
        },
        MaskComponentKind::LumaRange { min, max, softness } => {
            let lo = min.min(*max).clamp(0.0, 1.0);
            let hi = min.max(*max).clamp(0.0, 1.0);
            MaskComponentKind::LumaRange {
                min: lo,
                max: hi,
                softness: softness.clamp(0.0, 1.0),
            }
        }
        MaskComponentKind::ColorRange {
            sample_rgb,
            tolerance,
            softness,
        } => MaskComponentKind::ColorRange {
            sample_rgb: sample_rgb.map(|v| v.clamp(0.0, 1.0)),
            tolerance: tolerance.clamp(0.0, 1.0),
            softness: softness.clamp(0.0, 1.0),
        },
        MaskComponentKind::Polygon { points, feather } => MaskComponentKind::Polygon {
            points: points
                .iter()
                .take(N_MAX_POLYGON_POINTS)
                .map(|p| clamp_point(*p))
                .collect(),
            feather: feather.clamp(0.0, 1.0),
        },
    };
    MaskComponent {
        id: c.id.clone(),
        enabled: c.enabled,
        mode: c.mode,
        invert: c.invert,
        kind,
        source: c.source,
        generated: c.generated.as_ref().map(clamp_generated),
    }
}

pub(super) fn clamp_masks(layers: &[MaskLayer]) -> Vec<MaskLayer> {
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
            invert: layer.invert,
            components,
            edits: clamp_masked_edits(&layer.edits),
        });
    }
    out
}
