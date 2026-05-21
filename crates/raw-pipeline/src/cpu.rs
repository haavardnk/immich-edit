pub mod demosaic;
pub mod pipeline;
pub mod transform;

use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage, Renderer};

pub struct CpuRenderer;

impl Renderer for CpuRenderer {
    fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> crate::PipelineResult<RenderedImage> {
        pipeline::render(frame, edits, options)
    }

    fn name(&self) -> &str {
        "cpu"
    }
}
