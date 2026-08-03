// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat};

use crate::gpu::context::GpuContext;
use crate::gpu::shader_builder::{self, BuiltProcessShader};
use crate::ops::OpRegistry;

use super::common::{
    make_layout, make_pipeline_raw, storage_entry, tex_entry, uniform_entry_unsized,
};

pub struct WbPreparePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
    pub built: BuiltProcessShader,
}

impl WbPreparePass {
    pub fn new(ctx: &Arc<GpuContext>, registry: &OpRegistry) -> Self {
        let built = shader_builder::build_prepare_wb(registry);

        let layout = make_layout(
            ctx,
            "wb-prepare-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry(1),
                storage_entry(2, TextureFormat::Rgba16Float),
            ],
        );
        let pipeline = make_pipeline_raw(ctx, &layout, "wb-prepare-cp", &built.wgsl);
        Self {
            layout,
            pipeline,
            built,
        }
    }
}
