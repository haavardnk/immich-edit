use std::fmt::Write;

use crate::ops::OpRegistry;

pub const HEADER_BYTES: usize = 64;

pub struct ColorOpSlot {
    pub op_index: usize,
    pub uniform_offset: usize,
    pub vec4_count: usize,
}

pub struct BuiltProcessShader {
    pub wgsl: String,
    pub uniform_size: usize,
    pub color_ops: Vec<ColorOpSlot>,
}

pub fn build(registry: &OpRegistry) -> BuiltProcessShader {
    let mut struct_fields = String::new();
    let mut functions = String::new();
    let mut apply_calls = String::new();
    let mut color_ops: Vec<ColorOpSlot> = Vec::new();
    let mut used_vec4s: usize = 0;

    for (idx, op) in registry.ops().iter().enumerate() {
        let Some(gpu_op) = op.gpu() else { continue };
        if gpu_op.vec4_count == 0 {
            continue;
        }
        let offset = HEADER_BYTES + used_vec4s * 16;
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
        writeln!(apply_calls, "    {}", gpu_op.apply).unwrap();
        color_ops.push(ColorOpSlot {
            op_index: idx,
            uniform_offset: offset,
            vec4_count: gpu_op.vec4_count,
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
{struct_fields}}};

@group(0) @binding(0) var<uniform> p: ProcessParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var src_samp: sampler;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;

fn default_tone(v: f32) -> f32 {{
    let lin = clamp(v, 0.0, 1.0);
    var srgb: f32;
    if (lin <= 0.003130808) {{
        srgb = 12.92 * lin;
    }} else {{
        srgb = 1.055 * pow(lin, 1.0 / 2.4) - 0.055;
    }}
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    let out = srgb + (s - srgb) * 0.15;
    return clamp(out, 0.0, 1.0);
}}

{functions}
fn process_color(c0: vec3<f32>) -> vec3<f32> {{
    var lin = c0;
{apply_calls}    return lin;
}}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) {{ return; }}

    let ow = f32(p.out_size.x);
    let oh = f32(p.out_size.y);
    var u = (f32(gid.x) + 0.5) / ow;
    var v = (f32(gid.y) + 0.5) / oh;

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

    let rgb = textureSampleLevel(src_tex, src_samp, vec2<f32>(su, sv), 0.0).rgb;
    let outc_lin = process_color(rgb);
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
