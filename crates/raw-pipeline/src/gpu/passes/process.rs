// color-space: linear scene-referred Rgba16Float in; out is linear (full path) or sRGB tone-mapped (fast no-effects path, see process.wgsl)
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat};

use crate::gpu::context::GpuContext;
use crate::gpu::shader_builder::{self, BuiltProcessShader, StageMask};
use crate::ops::OpRegistry;

use super::common::{
    make_layout, make_pipeline_raw, sampler_entry, storage_entry, tex_entry, uniform_entry_unsized,
};

pub struct ProcessFastPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
    pub built: BuiltProcessShader,
}

impl ProcessFastPass {
    pub fn new(ctx: &Arc<GpuContext>, registry: &OpRegistry) -> Self {
        Self::new_with_mask(ctx, registry, StageMask::fast(), "process-fast")
    }

    pub fn new_with_mask(
        ctx: &Arc<GpuContext>,
        registry: &OpRegistry,
        mask: StageMask,
        label_prefix: &str,
    ) -> Self {
        let built = shader_builder::build_for(registry, mask);

        let layout = make_layout(
            ctx,
            &format!("{label_prefix}-bgl"),
            &[
                uniform_entry_unsized(0),
                tex_entry(1),
                sampler_entry(2),
                storage_entry(3, TextureFormat::Rgba8Unorm),
                storage_entry(4, TextureFormat::Rgba16Float),
                tex_entry(5),
            ],
        );
        let pipeline = make_pipeline_raw(ctx, &layout, &format!("{label_prefix}-cp"), &built.wgsl);
        Self {
            layout,
            pipeline,
            built,
        }
    }
}
