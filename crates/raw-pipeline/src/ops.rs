pub mod blur;
pub mod brightness;
pub mod capture_sharpen;
pub mod clarity;
pub mod color_grade;
pub mod color_matrix;
pub mod color_nr;
pub mod contrast;
pub mod curves;
pub mod dcp_profile;
pub mod dehaze;
pub mod exposure;
pub mod grain;
pub mod hsl;
pub mod lens_ca;
pub mod lens_distortion;
pub mod lens_profile;
pub mod lens_vignette;
pub mod luma_nr;
pub mod lut;
pub mod masks;
pub mod retouch;
pub mod sample;
pub mod saturation;
pub mod sharpen;
pub mod texture;
pub mod tone_regions;
pub mod transform;
pub mod user_wb;
pub mod vibrance;
pub mod vignette;
pub mod white_balance;

#[cfg(test)]
mod tests;

use crate::PipelineResult;
use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_segment};
use crate::edits::Edits;

pub struct LinearImage {
    pub rgb: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl LinearImage {
    pub fn new(rgb: Vec<f32>, width: usize, height: usize) -> Self {
        Self { rgb, width, height }
    }

    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Sensor,
    WhiteBalance,
    Tone,
    Color,
    Geometry,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuOpKind {
    Normal,
    Presence,
    Detail,
}

#[derive(Clone)]
pub struct RenderContext {
    pub wb_coeffs: [f32; 4],
    pub cam_to_srgb: [[f32; 3]; 3],
    pub is_raw: bool,
    pub capture_sigma: Option<f32>,
    pub preview_mode: crate::frame::PreviewMode,
    pub roi: Option<crate::edits::CropRect>,
    pub dcp: Option<std::sync::Arc<ResolvedDcp>>,
}

#[derive(Clone)]
pub struct ResolvedDcp {
    pub base_table: Option<std::sync::Arc<crate::dcp::HueSatMap>>,
    pub look_table: Option<std::sync::Arc<crate::dcp::HueSatMap>>,
    pub tone_curve: Option<std::sync::Arc<Vec<[f32; 2]>>>,
    pub to_pp: [[f32; 3]; 3],
    pub from_pp: [[f32; 3]; 3],
}

pub fn resolve_dcp(
    profile: &crate::dcp::DcpProfile,
    wb_coeffs: [f32; 4],
    edits: &crate::edits::DcpEdits,
) -> ([[f32; 3]; 3], ResolvedDcp) {
    let cam_to_srgb = crate::color::dcp_cam_to_srgb(profile, wb_coeffs, edits.illuminant);
    let g = crate::color::dcp_weight(profile, wb_coeffs, edits.illuminant);
    let base_table = if edits.use_base_table {
        match (&profile.huesatmap1, &profile.huesatmap2) {
            (Some(a), Some(b)) => Some(std::sync::Arc::new(crate::color::merge_huesat(
                a.as_ref(),
                b.as_ref(),
                g,
            ))),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        }
    } else {
        None
    };
    let look_table = if edits.use_look_table {
        profile.look_table.clone()
    } else {
        None
    };
    let tone_curve = if edits.use_tone_curve {
        if profile.has_tone_curve() {
            profile.tone_curve.clone()
        } else if profile.is_adobe() {
            Some(std::sync::Arc::new(
                crate::color::ACR_DEFAULT_TONE_CURVE.to_vec(),
            ))
        } else {
            None
        }
    } else {
        None
    };
    let resolved = ResolvedDcp {
        base_table,
        look_table,
        tone_curve,
        to_pp: crate::color::srgb_lin_to_prophoto_matrix(),
        from_pp: crate::color::prophoto_to_srgb_lin_matrix(),
    };
    (cam_to_srgb, resolved)
}

#[derive(Clone)]
pub struct SharpenDeltaMap {
    pub values: std::sync::Arc<Vec<f32>>,
    pub width: usize,
    pub height: usize,
}

impl SharpenDeltaMap {
    pub fn sample(&self, x: usize, y: usize, w: usize, h: usize) -> f32 {
        let sx = (x * self.width / w.max(1)).min(self.width.saturating_sub(1));
        let sy = (y * self.height / h.max(1)).min(self.height.saturating_sub(1));
        self.values[sy * self.width + sx]
    }
}

#[derive(Clone, Default)]
pub struct OpScratch {
    pub shadows_blur: Option<std::sync::Arc<Vec<f32>>>,
    pub sharpen_delta: Option<SharpenDeltaMap>,
}

#[derive(Clone)]
pub struct OpContext {
    pub render: RenderContext,
    pub scratch: OpScratch,
}

pub struct GpuOp {
    pub field_name: &'static str,
    pub functions: &'static str,
    pub apply: &'static str,
    pub vec4_count: usize,
    pub kind: GpuOpKind,
}

impl GpuOp {
    pub const fn new(
        field_name: &'static str,
        functions: &'static str,
        apply: &'static str,
    ) -> Self {
        Self {
            field_name,
            functions,
            apply,
            vec4_count: 1,
            kind: GpuOpKind::Normal,
        }
    }
}

pub trait Op: Send + Sync {
    fn id(&self) -> &'static str;
    fn stage(&self) -> Stage;
    fn order(&self) -> i32 {
        0
    }
    fn is_active(&self, edits: &Edits) -> bool;
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_doc(&self, _value: &serde_json::Value, _edits: &mut Edits) {}
    fn cpu_fused(&self, _edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        None
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        if let Some(op) = self.cpu_fused(edits, ctx) {
            let mut seg = FusedSegment::default();
            seg.push(op);
            apply_segment(image, &seg);
        }
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        None
    }
    fn gpu_kind(&self) -> GpuOpKind {
        GpuOpKind::Normal
    }
    fn write_gpu_uniform(&self, _edits: &Edits, _ctx: &OpContext, _dst: &mut [f32]) {}
}

pub struct OpRegistry {
    ops: Vec<Box<dyn Op>>,
}

impl OpRegistry {
    pub fn new(mut ops: Vec<Box<dyn Op>>) -> Self {
        ops.sort_by_key(|o| (o.stage(), o.order()));
        Self { ops }
    }

    pub fn ops(&self) -> &[Box<dyn Op>] {
        &self.ops
    }

    pub fn active<'a>(&'a self, edits: &'a Edits) -> impl Iterator<Item = &'a Box<dyn Op>> + 'a {
        self.ops.iter().filter(move |o| o.is_active(edits))
    }
}

pub fn default_registry() -> OpRegistry {
    OpRegistry::new(vec![
        Box::new(lens_profile::LensProfileOp),
        Box::new(lens_distortion::LensDistortionOp),
        Box::new(lens_vignette::LensVignetteOp),
        Box::new(lens_ca::LensCaOp),
        Box::new(white_balance::WhiteBalanceOp),
        Box::new(color_matrix::ColorMatrixOp),
        Box::new(user_wb::UserWbOp),
        Box::new(retouch::RetouchOp),
        Box::new(luma_nr::LumaNrOp),
        Box::new(color_nr::ColorNrOp),
        Box::new(capture_sharpen::CaptureSharpenOp),
        Box::new(texture::TextureOp),
        Box::new(clarity::ClarityOp),
        Box::new(dehaze::DehazeOp),
        Box::new(exposure::ExposureOp),
        Box::new(brightness::BrightnessOp),
        Box::new(tone_regions::ToneRegionsOp),
        Box::new(contrast::ContrastOp),
        Box::new(curves::CurvesOp),
        Box::new(saturation::SaturationOp),
        Box::new(vibrance::VibranceOp),
        Box::new(hsl::HslOp),
        Box::new(color_grade::ColorGradeOp),
        Box::new(dcp_profile::DcpProfileOp),
        Box::new(lut::Lut3dOp),
        Box::new(transform::TransformOp),
        Box::new(sharpen::SharpenOp),
        Box::new(vignette::VignetteOp),
        Box::new(grain::GrainOp),
        Box::new(masks::MasksOp),
    ])
}
