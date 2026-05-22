use wgpu::CommandEncoder;

use crate::PipelineResult;

pub trait GpuPass {
    fn label(&self) -> &'static str;
    fn encode(&self, encoder: &mut CommandEncoder) -> PipelineResult<()>;
}
