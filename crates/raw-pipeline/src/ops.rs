pub mod clarity;
pub mod color_grade;
pub mod color_matrix;
pub mod color_nr;
pub mod contrast;
pub mod crop_rotate;
pub mod curves;
pub mod dehaze;
pub mod exposure;
pub mod geometry;
pub mod grain;
pub mod hsl;
pub mod luma_nr;
pub mod masks;
pub mod saturation;
pub mod sharpen;
pub mod texture;
pub mod tone_regions;
pub mod user_wb;
pub mod vibrance;
pub mod vignette;
pub mod white_balance;

#[cfg(test)]
mod tests;

use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
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
pub enum ResourceNeed {
    LumaPyramid { max_radius_px: u32 },
}

#[derive(Clone)]
pub struct OpContext {
    pub wb_coeffs: [f32; 4],
    pub cam_to_srgb: [[f32; 3]; 3],
    pub is_raw: bool,
    pub preview_mode: crate::frame::PreviewMode,
    pub shadows_blur: Option<std::sync::Arc<Vec<f32>>>,
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
    fn is_active(&self, edits: &Edits) -> bool;
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()>;
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
    OpRegistry::new(vec![
        Box::new(white_balance::WhiteBalanceOp),
        Box::new(color_matrix::ColorMatrixOp),
        Box::new(user_wb::UserWbOp),
        Box::new(luma_nr::LumaNrOp),
        Box::new(color_nr::ColorNrOp),
        Box::new(texture::TextureOp),
        Box::new(clarity::ClarityOp),
        Box::new(dehaze::DehazeOp),
        Box::new(exposure::ExposureOp),
        Box::new(tone_regions::ToneRegionsOp),
        Box::new(contrast::ContrastOp),
        Box::new(curves::CurvesOp),
        Box::new(saturation::SaturationOp),
        Box::new(vibrance::VibranceOp),
        Box::new(hsl::HslOp),
        Box::new(color_grade::ColorGradeOp),
        Box::new(geometry::GeometryOp),
        Box::new(crop_rotate::CropRotateOp),
        Box::new(sharpen::SharpenOp),
        Box::new(vignette::VignetteOp),
        Box::new(grain::GrainOp),
        Box::new(masks::MasksOp),
    ])
}
