// color-space: linear scene-referred Rgba16Float in/out (blends per-mask op results)
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, ComputePipeline, StorageTextureAccess, TextureFormat, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline_raw, storage_entry, storage_entry_with, tex_entry_with,
    uniform_entry_unsized,
};

pub const PARAMS_BYTES: usize = 16;

const SHADER: &str = r#"
struct BlendParams {
    out_size: vec2<u32>,
    sharpen_delta: f32,
    sharpen_flags: u32,
};

@group(0) @binding(0) var<uniform> p: BlendParams;
@group(0) @binding(1) var curr_tex: texture_2d<f32>;
@group(0) @binding(2) var layer_tex: texture_2d<f32>;
@group(0) @binding(3) var weight_tex: texture_2d<f32>;
@group(0) @binding(4) var dst_tex: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var sharpen_tex: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(curr_tex, coord, 0).rgb;
    let l = textureLoad(layer_tex, coord, 0).rgb;
    let w = clamp(textureLoad(weight_tex, coord, 0).r, 0.0, 1.0);
    let outc = c + (l - c) * w;
    textureStore(dst_tex, coord, vec4<f32>(outc, 1.0));
    if (p.sharpen_flags == 1u) {
        textureStore(sharpen_tex, coord, vec4<f32>(w * p.sharpen_delta, 0.0, 0.0, 0.0));
    } else if (p.sharpen_flags == 2u) {
        let acc = textureLoad(sharpen_tex, coord).r + w * p.sharpen_delta;
        textureStore(sharpen_tex, coord, vec4<f32>(acc, 0.0, 0.0, 0.0));
    }
}
"#;

pub struct MaskBlendPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskBlendPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mask-blend-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D2),
                tex_entry_with(3, false, TextureViewDimension::D2),
                storage_entry(4, TextureFormat::Rgba16Float),
                storage_entry_with(5, TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
            ],
        );
        let pipeline = make_pipeline_raw(ctx, &layout, "mask-blend-cp", SHADER);
        Self { layout, pipeline }
    }
}

pub fn pack_params(
    out_w: u32,
    out_h: u32,
    sharpen_delta: f32,
    sharpen_flags: u32,
) -> [u8; PARAMS_BYTES] {
    let mut buf = [0u8; PARAMS_BYTES];
    buf[0..4].copy_from_slice(&out_w.to_ne_bytes());
    buf[4..8].copy_from_slice(&out_h.to_ne_bytes());
    buf[8..12].copy_from_slice(&sharpen_delta.to_ne_bytes());
    buf[12..16].copy_from_slice(&sharpen_flags.to_ne_bytes());
    buf
}
