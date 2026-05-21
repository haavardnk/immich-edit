pub mod context;
pub mod pipeline;
pub mod readback;
pub mod uniforms;

use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage, Renderer};
use crate::{PipelineError, PipelineResult};

pub struct GpuRenderer;

impl GpuRenderer {
    pub fn new() -> PipelineResult<Self> {
        Err(PipelineError::Unsupported("gpu renderer not yet implemented".into()))
    }
}

impl Renderer for GpuRenderer {
    fn render(
        &self,
        _frame: &RawFrame,
        _edits: &Edits,
        _options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        Err(PipelineError::Unsupported("gpu renderer not yet implemented".into()))
    }

    fn name(&self) -> &str {
        "gpu"
    }
}
