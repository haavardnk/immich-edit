struct Params {
    dst_size: vec2<u32>,
    src_size: vec2<u32>,
    scale: f32,
    inv_filter_scale: f32,
    axis: u32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

const PI: f32 = 3.14159265358979;

fn sinc(x: f32) -> f32 {
    if (abs(x) < 1e-6) { return 1.0; }
    let a = PI * x;
    return sin(a) / a;
}

fn lanczos3(x: f32) -> f32 {
    let a = abs(x);
    if (a >= 3.0) { return 0.0; }
    return sinc(a) * sinc(a * (1.0 / 3.0));
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.dst_size.x || gid.y >= p.dst_size.y) { return; }

    var idx = gid.x;
    var limit = i32(p.src_size.x) - 1;
    if (p.axis == 1u) {
        idx = gid.y;
        limit = i32(p.src_size.y) - 1;
    }

    let center = (f32(idx) + 0.5) * p.scale;
    let support = 3.0 / p.inv_filter_scale;
    let lo = max(i32(floor(center - support - 0.5)), 0);
    let hi = min(i32(ceil(center + support - 0.5)), limit);

    var acc = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var s = lo; s <= hi; s = s + 1) {
        let w = lanczos3((f32(s) + 0.5 - center) * p.inv_filter_scale);
        var c = vec2<i32>(s, i32(gid.y));
        if (p.axis == 1u) { c = vec2<i32>(i32(gid.x), s); }
        acc = acc + textureLoad(src, c, 0).rgb * w;
        wsum = wsum + w;
    }

    var outc = acc;
    if (abs(wsum) > 1e-8) { outc = acc / wsum; }
    textureStore(dst, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(max(outc, vec3<f32>(0.0)), 1.0));
}
