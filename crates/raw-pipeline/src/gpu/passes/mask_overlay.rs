// color-space: display-referred Rgba8Unorm in/out (tints the finished image by mask weight)
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry_unsized,
};

pub const PARAMS_BYTES: usize = size_of::<MaskOverlayParams>();
pub const OVERLAY_ALPHA: f32 = 0.55;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaskOverlayParams {
    pub out_size: [u32; 2],
    pub strength: f32,
    pub _pad: u32,
}

const SHADER: &str = r#"
struct OverlayParams {
    out_size: vec2<u32>,
    strength: f32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: OverlayParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var weight_tex: texture_2d<f32>;
@group(0) @binding(3) var dst_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = textureLoad(src_tex, coord, 0);
    let w = clamp(textureLoad(weight_tex, coord, 0).r, 0.0, 1.0);
    let a = w * p.strength;
    let outc = vec3<f32>(
        src.r + (1.0 - src.r) * a,
        src.g * (1.0 - a),
        src.b * (1.0 - a),
    );
    textureStore(dst_tex, coord, vec4<f32>(outc, src.a));
}
"#;

pub struct MaskOverlayPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskOverlayPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mask-overlay-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D2),
                storage_entry(3, TextureFormat::Rgba8Unorm),
            ],
        );
        let pipeline = make_pipeline_raw(ctx, &layout, "mask-overlay.wgsl", SHADER);
        Self { layout, pipeline }
    }
}

pub fn pack_params(out_w: u32, out_h: u32, strength: f32) -> MaskOverlayParams {
    MaskOverlayParams {
        out_size: [out_w, out_h],
        strength,
        _pad: 0,
    }
}
