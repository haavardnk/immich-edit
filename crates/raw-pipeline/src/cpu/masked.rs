mod blend;
mod weight;

#[cfg(test)]
mod tests;

use crate::edits::{Edits, MaskComponentKind, MaskComponentMode, MaskLayer};
use crate::mask_raster::{MaskRaster, RasterMap};
use std::sync::Arc;
use weight::display_srgb_to_oklab;

pub use blend::{blend_layer_images, build_sharpen_delta_image, render_mask_overlay};
pub use weight::{fold_layer_weight, fold_layer_weight_with_display};

#[derive(Clone, Debug)]
pub enum ComponentKindEval {
    Linear {
        p0: (f32, f32),
        dir: (f32, f32),
        len2: f32,
        feather: f32,
    },
    Radial {
        center: (f32, f32),
        inv_radius: (f32, f32),
        feather: f32,
    },
    Brush {
        raster_id: String,
        raster: Option<Arc<MaskRaster>>,
    },
    LumaRange {
        min: f32,
        max: f32,
        softness: f32,
    },
    ColorRange {
        sample_rgb: [f32; 3],
        sample_lab: [f32; 3],
        tolerance: f32,
        softness: f32,
    },
    Polygon {
        points: Vec<(f32, f32)>,
        feather: f32,
    },
}

#[derive(Clone, Debug)]
pub struct ComponentEval {
    pub mode: MaskComponentMode,
    pub invert: bool,
    pub kind: ComponentKindEval,
}

#[derive(Clone, Debug)]
pub struct LayerEval {
    pub amount: f32,
    pub invert: bool,
    pub components: Vec<ComponentEval>,
}

pub fn build_layer_evals(layers: &[MaskLayer], rasters: &RasterMap) -> Vec<LayerEval> {
    layers
        .iter()
        .filter(|l| l.is_effective())
        .map(|l| build_layer_eval(l, rasters))
        .collect()
}

pub fn build_layer_eval(layer: &MaskLayer, rasters: &RasterMap) -> LayerEval {
    let components: Vec<ComponentEval> = layer
        .components
        .iter()
        .filter(|c| c.enabled)
        .map(|c| {
            let kind = match &c.kind {
                MaskComponentKind::Linear { p0, p1, feather } => {
                    let dx = p1.x - p0.x;
                    let dy = p1.y - p0.y;
                    let len2 = (dx * dx + dy * dy).max(1e-12);
                    ComponentKindEval::Linear {
                        p0: (p0.x, p0.y),
                        dir: (dx, dy),
                        len2,
                        feather: feather.clamp(0.0, 1.0),
                    }
                }
                MaskComponentKind::Radial {
                    center,
                    radius_xy,
                    feather,
                } => {
                    let ix = if radius_xy.x.abs() < 1e-6 {
                        0.0
                    } else {
                        1.0 / radius_xy.x
                    };
                    let iy = if radius_xy.y.abs() < 1e-6 {
                        0.0
                    } else {
                        1.0 / radius_xy.y
                    };
                    ComponentKindEval::Radial {
                        center: (center.x, center.y),
                        inv_radius: (ix, iy),
                        feather: feather.clamp(0.0, 1.0),
                    }
                }
                MaskComponentKind::Brush { raster_id } => ComponentKindEval::Brush {
                    raster_id: raster_id.clone(),
                    raster: rasters.get(raster_id).cloned(),
                },
                MaskComponentKind::LumaRange { min, max, softness } => {
                    ComponentKindEval::LumaRange {
                        min: min.clamp(0.0, 1.0),
                        max: max.clamp(0.0, 1.0),
                        softness: softness.clamp(0.0, 1.0),
                    }
                }
                MaskComponentKind::ColorRange {
                    sample_rgb,
                    tolerance,
                    softness,
                } => ComponentKindEval::ColorRange {
                    sample_rgb: *sample_rgb,
                    sample_lab: display_srgb_to_oklab(*sample_rgb),
                    tolerance: tolerance.clamp(0.0, 1.0),
                    softness: softness.clamp(0.0, 1.0),
                },
                MaskComponentKind::Polygon { points, feather } => ComponentKindEval::Polygon {
                    points: points.iter().map(|p| (p.x, p.y)).collect(),
                    feather: feather.clamp(0.0, 1.0),
                },
            };
            ComponentEval {
                mode: c.mode,
                invert: c.invert,
                kind,
            }
        })
        .collect();
    LayerEval {
        amount: layer.amount.clamp(0.0, 1.0),
        invert: layer.invert,
        components,
    }
}
pub fn effective_edits_for_layer(global: &Edits, layer: &MaskLayer) -> Edits {
    let mut out = global.clone();
    let d = &layer.edits;
    if let Some(v) = d.exposure_ev {
        out.basic.exposure_ev = (out.basic.exposure_ev + v).clamp(-5.0, 5.0);
    }
    if let Some(v) = d.brightness {
        out.basic.brightness = (out.basic.brightness + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.contrast {
        out.basic.contrast = (out.basic.contrast + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.saturation {
        out.basic.saturation = (out.basic.saturation + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.vibrance {
        out.basic.vibrance = (out.basic.vibrance + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.wb_temp {
        out.basic.wb_temp = (out.basic.wb_temp + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.wb_tint {
        out.basic.wb_tint = (out.basic.wb_tint + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.highlights {
        out.tone.highlights = (out.tone.highlights + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.shadows {
        out.tone.shadows = (out.tone.shadows + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.whites {
        out.tone.whites = (out.tone.whites + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.blacks {
        out.tone.blacks = (out.tone.blacks + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.texture {
        out.basic.texture = (out.basic.texture + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.clarity {
        out.basic.clarity = (out.basic.clarity + v).clamp(-100.0, 100.0);
    }
    out
}
