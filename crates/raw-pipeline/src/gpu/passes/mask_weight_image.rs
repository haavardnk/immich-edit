// color-space: mask weight in, Rgba8Unorm gray weight image out (R = G = B = weight)
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry_unsized,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaskWeightImageParams {
    pub out_size: [u32; 2],
    pub _pad: [u32; 2],
}

pub const MASK_WEIGHT_IMAGE_WGSL: &str = r#"
struct WeightImageParams {
    out_size: vec2<u32>,
};

@group(0) @binding(0) var<uniform> p: WeightImageParams;
@group(0) @binding(1) var weight_tex: texture_2d<f32>;
@group(0) @binding(2) var dst_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let w = clamp(textureLoad(weight_tex, coord, 0).r, 0.0, 1.0);
    textureStore(dst_tex, coord, vec4<f32>(w, w, w, 1.0));
}
"#;

pub struct MaskWeightImagePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskWeightImagePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mask-weight-image-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry_with(1, false, TextureViewDimension::D2),
                storage_entry(2, TextureFormat::Rgba8Unorm),
            ],
        );
        let pipeline = make_pipeline_raw(
            ctx,
            &layout,
            "mask-weight-image.wgsl",
            MASK_WEIGHT_IMAGE_WGSL,
        );
        Self { layout, pipeline }
    }
}

pub fn pack_params(out_w: u32, out_h: u32) -> MaskWeightImageParams {
    MaskWeightImageParams {
        out_size: [out_w, out_h],
        _pad: [0; 2],
    }
}
