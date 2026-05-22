struct PresenceParams {
    size: vec2<u32>,
    amounts: vec4<f32>,
    mips: vec4<u32>,
}

@group(0) @binding(0) var<uniform> p: PresenceParams;
@group(0) @binding(1) var base: texture_2d<f32>;
@group(0) @binding(2) var pyramid: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

fn luma_at(level: u32, x: i32, y: i32) -> f32 {
    let dim = textureDimensions(pyramid, level);
    let ix = clamp(x, 0, i32(dim.x) - 1);
    let iy = clamp(y, 0, i32(dim.y) - 1);
    return textureLoad(pyramid, vec2<i32>(ix, iy), i32(level)).r;
}

fn sampled_luma(level: u32, fx: f32, fy: f32) -> f32 {
    let scale = 1.0 / f32(1u << level);
    let lx = fx * scale - 0.5;
    let ly = fy * scale - 0.5;
    let x0 = i32(floor(lx));
    let y0 = i32(floor(ly));
    let tx = lx - f32(x0);
    let ty = ly - f32(y0);
    let l00 = luma_at(level, x0, y0);
    let l10 = luma_at(level, x0 + 1, y0);
    let l01 = luma_at(level, x0, y0 + 1);
    let l11 = luma_at(level, x0 + 1, y0 + 1);
    let lx0 = mix(l00, l10, tx);
    let lx1 = mix(l01, l11, tx);
    return mix(lx0, lx1, ty);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let rgb = textureLoad(base, pos, 0).rgb;
    let y0 = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;

    let fx = f32(gid.x) + 0.5;
    let fy = f32(gid.y) + 0.5;

    var delta: f32 = 0.0;
    if (p.amounts.x != 0.0) {
        let blurred = sampled_luma(p.mips.x, fx, fy);
        delta = delta + p.amounts.x * (y0 - blurred);
    }
    if (p.amounts.y != 0.0) {
        let blurred = sampled_luma(p.mips.y, fx, fy);
        delta = delta + p.amounts.y * (y0 - blurred);
    }
    if (p.amounts.z != 0.0) {
        let blurred = sampled_luma(p.mips.z, fx, fy);
        delta = delta + p.amounts.z * (y0 - blurred);
    }

    let goal = max(y0 + delta, 0.0);
    var scale: f32 = 1.0;
    if (y0 > 1e-5) { scale = goal / y0; }
    let outc = rgb * scale;
    textureStore(dst, pos, vec4<f32>(outc, 1.0));
}
