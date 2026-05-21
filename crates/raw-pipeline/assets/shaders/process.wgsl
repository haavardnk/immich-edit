struct ProcessParams {
    src_size: vec2<u32>,
    out_size: vec2<u32>,
    crop: vec4<f32>,
    wb: vec4<f32>,
    tone: vec4<f32>,
    flags: vec4<u32>,
    sat: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> p: ProcessParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var src_samp: sampler;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;

fn srgb_encode(v: f32) -> f32 {
    let x = clamp(v, 0.0, 1.0);
    if (x <= 0.0031308) { return x * 12.92; }
    return 1.055 * pow(x, 1.0 / 2.4) - 0.055;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }

    let ow = f32(p.out_size.x);
    let oh = f32(p.out_size.y);

    var u = (f32(gid.x) + 0.5) / ow;
    var v = (f32(gid.y) + 0.5) / oh;

    let rot = p.flags.x;
    let flip_h = p.flags.y;
    let flip_v = p.flags.z;

    var cu = p.crop.x + u * p.crop.z;
    var cv = p.crop.y + v * p.crop.w;

    if (flip_h == 1u) { cu = 1.0 - cu; }
    if (flip_v == 1u) { cv = 1.0 - cv; }

    var su: f32;
    var sv: f32;
    if (rot == 90u) { su = cv; sv = 1.0 - cu; }
    else if (rot == 180u) { su = 1.0 - cu; sv = 1.0 - cv; }
    else if (rot == 270u) { su = 1.0 - cv; sv = cu; }
    else { su = cu; sv = cv; }

    let orient = p.flags.w;
    let oh_h = (orient & 1u) != 0u;
    let oh_v = (orient & 2u) != 0u;
    let oh_t = (orient & 4u) != 0u;
    if (oh_t) { let tmp = su; su = sv; sv = tmp; }
    if (oh_v) { sv = 1.0 - sv; }
    if (oh_h) { su = 1.0 - su; }

    let rgb = textureSampleLevel(src_tex, src_samp, vec2<f32>(su, sv), 0.0).rgb;

    var lin = vec3<f32>(rgb.r * p.wb.r, rgb.g * p.wb.g, rgb.b * p.wb.b);

    let exposure = p.tone.x;
    let contrast = p.tone.y;
    let hl = p.tone.z;
    let sh = p.tone.w;

    lin = lin * exposure;

    if (hl != 0.0 || sh != 0.0) {
        var out_v = vec3<f32>(0.0);
        for (var i = 0u; i < 3u; i = i + 1u) {
            let x = clamp(lin[i], 0.0, 2.0);
            if (x > 0.5) { out_v[i] = x + hl * (1.0 - x) * 0.5; }
            else { out_v[i] = x + sh * x * 0.5; }
        }
        lin = out_v;
    }

    if (contrast != 0.0) {
        let f = 1.0 + contrast;
        lin = (lin - vec3<f32>(0.5)) * f + vec3<f32>(0.5);
    }

    if (p.sat != 0.0) {
        let f = 1.0 + p.sat;
        let luma = 0.2126 * lin.r + 0.7152 * lin.g + 0.0722 * lin.b;
        lin = vec3<f32>(luma) + (lin - vec3<f32>(luma)) * f;
    }

    let outc = vec3<f32>(srgb_encode(lin.r), srgb_encode(lin.g), srgb_encode(lin.b));
    textureStore(out_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(outc, 1.0));
}
