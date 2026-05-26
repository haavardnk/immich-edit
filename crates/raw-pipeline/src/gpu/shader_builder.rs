use std::fmt::Write;

use crate::ops::{OpRegistry, Stage};

pub const HEADER_BYTES: usize = 112;
pub const ACTIVE_MASK_OFFSET: usize = 64;
pub const MAX_OPS: u32 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageMask(u8);

impl StageMask {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn with(mut self, stage: Stage) -> Self {
        self.0 |= 1 << stage as u8;
        self
    }

    pub const fn fast() -> Self {
        Self(
            (1 << Stage::WhiteBalance as u8) | (1 << Stage::Tone as u8) | (1 << Stage::Color as u8),
        )
    }

    pub const fn white_balance() -> Self {
        Self(1 << Stage::WhiteBalance as u8)
    }

    pub const fn tone_color() -> Self {
        Self((1 << Stage::Tone as u8) | (1 << Stage::Color as u8))
    }

    pub fn contains(self, stage: Stage) -> bool {
        (self.0 >> stage as u8) & 1 != 0
    }
}

pub struct ColorOpSlot {
    pub op_index: usize,
    pub uniform_offset: usize,
    pub vec4_count: usize,
    pub active_bit: u32,
}

pub struct BuiltProcessShader {
    pub wgsl: String,
    pub uniform_size: usize,
    pub color_ops: Vec<ColorOpSlot>,
}

pub fn build(registry: &OpRegistry) -> BuiltProcessShader {
    build_for(registry, StageMask::fast())
}

pub fn build_for(registry: &OpRegistry, mask: StageMask) -> BuiltProcessShader {
    let mut struct_fields = String::new();
    let mut functions = String::new();
    let mut apply_wb = String::new();
    let mut apply_tone = String::new();
    let mut apply_color = String::new();
    let mut color_ops: Vec<ColorOpSlot> = Vec::new();
    let mut used_vec4s: usize = 0;

    for (idx, op) in registry.ops().iter().enumerate() {
        let Some(gpu_op) = op.gpu() else { continue };
        if gpu_op.vec4_count == 0 {
            continue;
        }
        let stage = op.stage();
        if !mask.contains(stage) {
            continue;
        }
        let offset = HEADER_BYTES + used_vec4s * 16;
        let bit = color_ops.len() as u32;
        if bit >= MAX_OPS {
            panic!("more than {MAX_OPS} GPU ops; active_mask layout needs expansion");
        }
        if gpu_op.vec4_count == 1 {
            writeln!(struct_fields, "    {}: vec4<f32>,", gpu_op.field_name).unwrap();
        } else {
            writeln!(
                struct_fields,
                "    {}: array<vec4<f32>, {}>,",
                gpu_op.field_name, gpu_op.vec4_count
            )
            .unwrap();
        }
        functions.push_str(gpu_op.functions);
        functions.push('\n');
        let chunk = match stage {
            Stage::WhiteBalance => &mut apply_wb,
            Stage::Tone => &mut apply_tone,
            Stage::Color => &mut apply_color,
            Stage::Sensor => unreachable!("sensor ops use vec4_count == 0"),
            Stage::Geometry => unreachable!("geometry ops use vec4_count == 0"),
            Stage::Output => unreachable!("output ops use vec4_count == 0"),
        };
        writeln!(chunk, "    if (is_active({bit}u)) {{ {} }}", gpu_op.apply).unwrap();
        color_ops.push(ColorOpSlot {
            op_index: idx,
            uniform_offset: offset,
            vec4_count: gpu_op.vec4_count,
            active_bit: bit,
        });
        used_vec4s += gpu_op.vec4_count;
    }

    let uniform_size = HEADER_BYTES + used_vec4s * 16;

    let process_chain = build_process_chain(mask);

    let wgsl = format!(
        r#"struct ProcessParams {{
    src_size: vec2<u32>,
    out_size: vec2<u32>,
    crop: vec4<f32>,
    flags: vec4<u32>,
    geom_extra: vec4<f32>,
    active_mask: vec4<u32>,
    geom_extra2: vec4<f32>,
    geom_extra3: vec4<f32>,
{struct_fields}}};

@group(0) @binding(0) var<uniform> p: ProcessParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var src_samp: sampler;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var linear_tex: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var shadows_blur_tex: texture_2d<f32>;

var<private> shadows_blur_l: f32 = 0.0;

fn is_active(bit: u32) -> bool {{
    let word = bit / 32u;
    let shift = bit % 32u;
    return ((p.active_mask[word] >> shift) & 1u) != 0u;
}}

fn soft_clip_high(v: f32) -> f32 {{
    let knee: f32 = 0.95;
    if (v <= knee) {{ return v; }}
    let headroom: f32 = 1.0 - knee;
    let excess: f32 = v - knee;
    return knee + headroom * (excess / (excess + headroom));
}}

fn default_tone(v: f32) -> f32 {{
    var lin: f32;
    if (v <= 0.0) {{ lin = 0.0; }} else {{ lin = soft_clip_high(v); }}
    var srgb: f32;
    if (lin <= 0.003130808) {{
        srgb = 12.92 * lin;
    }} else {{
        srgb = 1.055 * pow(lin, 1.0 / 2.4) - 0.055;
    }}
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * 0.15;
}}

{functions}
fn apply_wb_stage(c: vec3<f32>) -> vec3<f32> {{
    var lin = c;
{apply_wb}    return lin;
}}

fn apply_tone_stage(c: vec3<f32>) -> vec3<f32> {{
    var lin = c;
{apply_tone}    return lin;
}}

fn apply_color_stage(c: vec3<f32>) -> vec3<f32> {{
    var lin = c;
{apply_color}    return lin;
}}

fn process_color(c0: vec3<f32>) -> vec3<f32> {{
    return {process_chain};
}}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) {{ return; }}

    let ow = f32(p.out_size.x);
    let oh = f32(p.out_size.y);
    var u = (f32(gid.x) + 0.5) / ow;
    var v = (f32(gid.y) + 0.5) / oh;

    let bx_rel = p.crop.x + u * p.crop.z;
    let by_rel = p.crop.y + v * p.crop.w;
    let cx_px = (bx_rel - 0.5) * p.geom_extra2.z;
    let cy_px = (by_rel - 0.5) * p.geom_extra2.w;
    let sx_px = cx_px * p.geom_extra2.x + cy_px * p.geom_extra2.y;
    let sy_px = -cx_px * p.geom_extra2.y + cy_px * p.geom_extra2.x;
    u = sx_px / p.geom_extra3.x + 0.5;
    v = sy_px / p.geom_extra3.y + 0.5;

    let rot = p.flags.x;
    let flip_h = p.flags.y;
    let flip_v = p.flags.z;

    var cu = u;
    var cv = v;

    if (flip_h == 1u) {{ cu = 1.0 - cu; }}
    if (flip_v == 1u) {{ cv = 1.0 - cv; }}

    var su: f32;
    var sv: f32;
    if (rot == 90u) {{ su = cv; sv = 1.0 - cu; }}
    else if (rot == 180u) {{ su = 1.0 - cu; sv = 1.0 - cv; }}
    else if (rot == 270u) {{ su = 1.0 - cv; sv = cu; }}
    else {{ su = cu; sv = cv; }}

    let orient = p.flags.w;
    let oh_h = (orient & 1u) != 0u;
    let oh_v = (orient & 2u) != 0u;
    let oh_t = (orient & 4u) != 0u;
    if (oh_t) {{ let tmp = su; su = sv; sv = tmp; }}
    if (oh_v) {{ sv = 1.0 - sv; }}
    if (oh_h) {{ su = 1.0 - su; }}

    let rgb = textureSampleLevel(src_tex, src_samp, vec2<f32>(su, sv), p.geom_extra.x).rgb;
    shadows_blur_l = textureSampleLevel(shadows_blur_tex, src_samp, vec2<f32>(su, sv), p.geom_extra.y).r;
    let outc_lin = process_color(rgb);
    textureStore(linear_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(outc_lin, 1.0));
    let outc = vec3<f32>(default_tone(outc_lin.r), default_tone(outc_lin.g), default_tone(outc_lin.b));
    textureStore(out_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(outc, 1.0));
}}
"#
    );

    BuiltProcessShader {
        wgsl,
        uniform_size,
        color_ops,
    }
}

pub fn build_prepare_wb(registry: &OpRegistry) -> BuiltProcessShader {
    let mask = StageMask::white_balance();
    let mut struct_fields = String::new();
    let mut functions = String::new();
    let mut apply_wb = String::new();
    let mut color_ops: Vec<ColorOpSlot> = Vec::new();
    let mut used_vec4s: usize = 0;

    for (idx, op) in registry.ops().iter().enumerate() {
        let Some(gpu_op) = op.gpu() else { continue };
        if gpu_op.vec4_count == 0 {
            continue;
        }
        let stage = op.stage();
        if !mask.contains(stage) {
            continue;
        }
        let offset = HEADER_BYTES + used_vec4s * 16;
        let bit = color_ops.len() as u32;
        if gpu_op.vec4_count == 1 {
            writeln!(struct_fields, "    {}: vec4<f32>,", gpu_op.field_name).unwrap();
        } else {
            writeln!(
                struct_fields,
                "    {}: array<vec4<f32>, {}>,",
                gpu_op.field_name, gpu_op.vec4_count
            )
            .unwrap();
        }
        functions.push_str(gpu_op.functions);
        functions.push('\n');
        writeln!(
            apply_wb,
            "    if (is_active({bit}u)) {{ {} }}",
            gpu_op.apply
        )
        .unwrap();
        color_ops.push(ColorOpSlot {
            op_index: idx,
            uniform_offset: offset,
            vec4_count: gpu_op.vec4_count,
            active_bit: bit,
        });
        used_vec4s += gpu_op.vec4_count;
    }

    let uniform_size = HEADER_BYTES + used_vec4s * 16;

    let wgsl = format!(
        r#"struct ProcessParams {{
    src_size: vec2<u32>,
    out_size: vec2<u32>,
    crop: vec4<f32>,
    flags: vec4<u32>,
    geom_extra: vec4<f32>,
    active_mask: vec4<u32>,
    geom_extra2: vec4<f32>,
    geom_extra3: vec4<f32>,
{struct_fields}}};

@group(0) @binding(0) var<uniform> p: ProcessParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var dst_tex: texture_storage_2d<rgba16float, write>;

fn is_active(bit: u32) -> bool {{
    let word = bit / 32u;
    let shift = bit % 32u;
    return ((p.active_mask[word] >> shift) & 1u) != 0u;
}}

{functions}
fn apply_wb_stage(c: vec3<f32>) -> vec3<f32> {{
    var lin = c;
{apply_wb}    return lin;
}}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    if (gid.x >= p.src_size.x || gid.y >= p.src_size.y) {{ return; }}
    let c = textureLoad(src_tex, vec2<i32>(i32(gid.x), i32(gid.y)), 0).rgb;
    let outc = apply_wb_stage(c);
    textureStore(dst_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(outc, 1.0));
}}
"#
    );

    BuiltProcessShader {
        wgsl,
        uniform_size,
        color_ops,
    }
}

fn build_process_chain(mask: StageMask) -> String {
    let stages = [
        (Stage::WhiteBalance, "apply_wb_stage"),
        (Stage::Tone, "apply_tone_stage"),
        (Stage::Color, "apply_color_stage"),
    ];
    let mut expr = String::from("c0");
    for (stage, name) in stages {
        if mask.contains(stage) {
            expr = format!("{name}({expr})");
        }
    }
    expr
}
