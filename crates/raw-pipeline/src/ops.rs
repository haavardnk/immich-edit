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

pub trait EditOperator: Send + Sync {
    fn id(&self) -> &'static str;
    fn stage(&self) -> Stage;
    fn order(&self) -> i32 {
        0
    }
    fn kind(&self) -> OpKind {
        OpKind::Fused
    }
    fn is_active(&self, edits: &Edits) -> bool;
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        if matches!(self.kind(), OpKind::Spatial) {
            panic!(
                "{}: OpKind::Spatial requires apply_cpu override",
                self.id()
            );
        }
        if let Some(op) = self.cpu_fused(edits, ctx) {
            let mut seg = FusedSegment::default();
            seg.push(op);
            apply_segment(image, &seg);
        }
        Ok(())
    }
    fn cpu_fused(&self, _edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        None
    }
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
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_doc(&self, _value: &serde_json::Value, _edits: &mut Edits) {}
}

pub struct OpRegistry {
    ops: Vec<Box<dyn EditOperator>>,
}

impl OpRegistry {
    pub fn new(mut ops: Vec<Box<dyn EditOperator>>) -> Self {
        ops.sort_by_key(|o| (o.stage(), o.order()));
        Self { ops }
    }

    pub fn ops(&self) -> &[Box<dyn EditOperator>] {
        &self.ops
    }

    pub fn active<'a>(
        &'a self,
        edits: &'a Edits,
    ) -> impl Iterator<Item = &'a dyn EditOperator> + 'a {
        self.ops
            .iter()
            .filter(move |o| o.is_active(edits))
            .map(|o| o.as_ref())
    }
}

pub fn default_registry() -> OpRegistry {
    let registry = OpRegistry::new(vec![
        Box::new(lens_profile::LensProfileOp),
        Box::new(lens_distortion::LensDistortionOp),
        Box::new(lens_vignette::LensVignetteOp),
        Box::new(lens_ca::LensCaOp),
        Box::new(white_balance::WhiteBalanceOp),
        Box::new(color_matrix::ColorMatrixOp),
        Box::new(user_wb::UserWbOp),
        Box::new(luma_nr::LumaNrOp),
        Box::new(color_nr::ColorNrOp),
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
        Box::new(transform::TransformOp),
        Box::new(sharpen::SharpenOp),
        Box::new(vignette::VignetteOp),
        Box::new(grain::GrainOp),
        Box::new(masks::MasksOp),
        Box::new(output::OutputOp),
    ]);
    for op in registry.ops() {
        if matches!(op.kind(), OpKind::Output) && op.stage() != Stage::Output {
            panic!("{}: OpKind::Output requires Stage::Output", op.id());
        }
    }
    let output_kinds = registry
        .ops()
        .iter()
        .filter(|o| matches!(o.kind(), OpKind::Output))
        .count();
    if output_kinds != 1 {
        panic!("expected exactly one OpKind::Output, found {output_kinds}");
    }
    registry
}
