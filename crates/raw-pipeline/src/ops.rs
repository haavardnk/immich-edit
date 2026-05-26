pub mod brightness;
pub mod clarity;
pub mod color_grade;
pub mod color_matrix;
pub mod color_nr;
pub mod contrast;
pub mod curves;
pub mod dehaze;
pub mod exposure;
pub mod grain;
pub mod hsl;
pub mod lens_ca;
pub mod lens_distortion;
pub mod lens_profile;
pub mod lens_vignette;
pub mod luma_nr;
pub mod masks;
pub mod output;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    Fused,
    Spatial,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceNeed {
    LumaPyramid { max_radius_px: u32 },
}

#[derive(Clone)]
pub struct RenderContext {
    pub wb_coeffs: [f32; 4],
    pub cam_to_srgb: [[f32; 3]; 3],
    pub is_raw: bool,
    pub preview_mode: crate::frame::PreviewMode,
}

#[derive(Clone, Default)]
pub struct OpScratch {
    pub shadows_blur: Option<std::sync::Arc<Vec<f32>>>,
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

pub trait OpMeta: Send + Sync {
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
}

pub trait FusedOp: OpMeta {
    fn cpu_fused(&self, edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp>;
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

pub trait SpatialOp: OpMeta {
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()>;
    fn gpu(&self) -> Option<GpuOp> {
        None
    }
    fn gpu_kind(&self) -> GpuOpKind {
        GpuOpKind::Normal
    }
    fn resource_needs(&self, _edits: &Edits) -> Vec<ResourceNeed> {
        Vec::new()
    }
    fn write_gpu_uniform(&self, _edits: &Edits, _ctx: &OpContext, _dst: &mut [f32]) {}
}

pub trait OutputStageOp: OpMeta {
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()>;
}

pub enum AnyOp {
    Fused(Box<dyn FusedOp>),
    Spatial(Box<dyn SpatialOp>),
    Output(Box<dyn OutputStageOp>),
}

impl AnyOp {
    pub fn kind(&self) -> OpKind {
        match self {
            AnyOp::Fused(_) => OpKind::Fused,
            AnyOp::Spatial(_) => OpKind::Spatial,
            AnyOp::Output(_) => OpKind::Output,
        }
    }
    pub fn id(&self) -> &'static str {
        match self {
            AnyOp::Fused(o) => o.id(),
            AnyOp::Spatial(o) => o.id(),
            AnyOp::Output(o) => o.id(),
        }
    }
    pub fn stage(&self) -> Stage {
        match self {
            AnyOp::Fused(o) => o.stage(),
            AnyOp::Spatial(o) => o.stage(),
            AnyOp::Output(o) => o.stage(),
        }
    }
    pub fn order(&self) -> i32 {
        match self {
            AnyOp::Fused(o) => o.order(),
            AnyOp::Spatial(o) => o.order(),
            AnyOp::Output(o) => o.order(),
        }
    }
    pub fn is_active(&self, edits: &Edits) -> bool {
        match self {
            AnyOp::Fused(o) => o.is_active(edits),
            AnyOp::Spatial(o) => o.is_active(edits),
            AnyOp::Output(o) => o.is_active(edits),
        }
    }
    pub fn cpu_fused(&self, edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        if let AnyOp::Fused(o) = self {
            o.cpu_fused(edits, ctx)
        } else {
            None
        }
    }
    pub fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        match self {
            AnyOp::Fused(o) => {
                if let Some(op) = o.cpu_fused(edits, ctx) {
                    let mut seg = FusedSegment::default();
                    seg.push(op);
                    apply_segment(image, &seg);
                }
                Ok(())
            }
            AnyOp::Spatial(o) => o.apply_cpu(image, ctx, edits),
            AnyOp::Output(o) => o.apply_cpu(image, ctx, edits),
        }
    }
    pub fn gpu(&self) -> Option<GpuOp> {
        match self {
            AnyOp::Fused(o) => o.gpu(),
            AnyOp::Spatial(o) => o.gpu(),
            AnyOp::Output(_) => None,
        }
    }
    pub fn gpu_kind(&self) -> GpuOpKind {
        match self {
            AnyOp::Fused(o) => o.gpu_kind(),
            AnyOp::Spatial(o) => o.gpu_kind(),
            AnyOp::Output(_) => GpuOpKind::Normal,
        }
    }
    pub fn resource_needs(&self, edits: &Edits) -> Vec<ResourceNeed> {
        if let AnyOp::Spatial(o) = self {
            o.resource_needs(edits)
        } else {
            Vec::new()
        }
    }
    pub fn write_gpu_uniform(&self, edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        match self {
            AnyOp::Fused(o) => o.write_gpu_uniform(edits, ctx, dst),
            AnyOp::Spatial(o) => o.write_gpu_uniform(edits, ctx, dst),
            AnyOp::Output(_) => {}
        }
    }
    pub fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        match self {
            AnyOp::Fused(o) => o.to_doc(edits),
            AnyOp::Spatial(o) => o.to_doc(edits),
            AnyOp::Output(o) => o.to_doc(edits),
        }
    }
    pub fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        match self {
            AnyOp::Fused(o) => o.from_doc(value, edits),
            AnyOp::Spatial(o) => o.from_doc(value, edits),
            AnyOp::Output(o) => o.from_doc(value, edits),
        }
    }
}

pub struct OpRegistry {
    ops: Vec<AnyOp>,
}

impl OpRegistry {
    pub fn new(mut ops: Vec<AnyOp>) -> Self {
        ops.sort_by_key(|o| (o.stage(), o.order()));
        Self { ops }
    }

    pub fn ops(&self) -> &[AnyOp] {
        &self.ops
    }

    pub fn active<'a>(&'a self, edits: &'a Edits) -> impl Iterator<Item = &'a AnyOp> + 'a {
        self.ops.iter().filter(move |o| o.is_active(edits))
    }
}

pub fn default_registry() -> OpRegistry {
    OpRegistry::new(vec![
        AnyOp::Spatial(Box::new(lens_profile::LensProfileOp)),
        AnyOp::Spatial(Box::new(lens_distortion::LensDistortionOp)),
        AnyOp::Spatial(Box::new(lens_vignette::LensVignetteOp)),
        AnyOp::Spatial(Box::new(lens_ca::LensCaOp)),
        AnyOp::Fused(Box::new(white_balance::WhiteBalanceOp)),
        AnyOp::Fused(Box::new(color_matrix::ColorMatrixOp)),
        AnyOp::Fused(Box::new(user_wb::UserWbOp)),
        AnyOp::Spatial(Box::new(luma_nr::LumaNrOp)),
        AnyOp::Spatial(Box::new(color_nr::ColorNrOp)),
        AnyOp::Spatial(Box::new(texture::TextureOp)),
        AnyOp::Spatial(Box::new(clarity::ClarityOp)),
        AnyOp::Spatial(Box::new(dehaze::DehazeOp)),
        AnyOp::Fused(Box::new(exposure::ExposureOp)),
        AnyOp::Fused(Box::new(brightness::BrightnessOp)),
        AnyOp::Fused(Box::new(tone_regions::ToneRegionsOp)),
        AnyOp::Fused(Box::new(contrast::ContrastOp)),
        AnyOp::Fused(Box::new(curves::CurvesOp)),
        AnyOp::Fused(Box::new(saturation::SaturationOp)),
        AnyOp::Fused(Box::new(vibrance::VibranceOp)),
        AnyOp::Fused(Box::new(hsl::HslOp)),
        AnyOp::Fused(Box::new(color_grade::ColorGradeOp)),
        AnyOp::Spatial(Box::new(transform::TransformOp)),
        AnyOp::Spatial(Box::new(sharpen::SharpenOp)),
        AnyOp::Spatial(Box::new(vignette::VignetteOp)),
        AnyOp::Spatial(Box::new(grain::GrainOp)),
        AnyOp::Spatial(Box::new(masks::MasksOp)),
        AnyOp::Output(Box::new(output::OutputOp)),
    ])
}
