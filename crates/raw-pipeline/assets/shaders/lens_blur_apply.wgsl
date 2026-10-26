@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(3) var grid: texture_2d<f32>;
@group(0) @binding(4) var blurred: texture_2d<f32>;
@group(0) @binding(5) var dst: texture_storage_2d<rgba16float, write>;

fn upsample_weight(cell: f32, pixel: f32) -> f32 {
    return 1.0 / (1.0 + UPSAMPLE_SHARPNESS * abs(cell - pixel));
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) {
        return;
    }
    let local = vec2<i32>(gid.xy);
    let sharp = textureLoad(src, local, 0);
    let o = sensor_to_oriented(gid.xy + p.origin);
    let pixel = signed_coc_at(o);
    let ds = f32(p.ds);
    let gw = i32(p.grid_size.x);
    let gh = i32(p.grid_size.y);
    let gx0 = i32(p.grid_origin.x);
    let gy0 = i32(p.grid_origin.y);
    let fx = (f32(o.x) + 0.5) / ds - 0.5;
    let fy = (f32(o.y) + 0.5) / ds - 0.5;
    let x0 = floor(fx);
    let y0 = floor(fy);
    let tx = fx - x0;
    let ty = fy - y0;
    let cols = vec2<i32>(
        clamp(i32(x0) - gx0, 0, gw - 1),
        clamp(i32(x0) + 1 - gx0, 0, gw - 1),
    );
    let rows = vec2<i32>(
        clamp(i32(y0) - gy0, 0, gh - 1),
        clamp(i32(y0) + 1 - gy0, 0, gh - 1),
    );
    let wxs = vec2<f32>(1.0 - tx, tx);
    let wys = vec2<f32>(1.0 - ty, ty);
    var acc = vec4<f32>(0.0);
    var total = 0.0;
    for (var ry = 0; ry < 2; ry = ry + 1) {
        for (var cx = 0; cx < 2; cx = cx + 1) {
            let cell = vec2<i32>(cols[cx], rows[ry]);
            let weight = wxs[cx] * wys[ry]
                * upsample_weight(textureLoad(grid, cell, 0).a, pixel / ds);
            acc = acc + textureLoad(blurred, cell, 0) * weight;
            total = total + weight;
        }
    }
    let mix_t = smoothstep(0.0, 1.0, max(abs(pixel) / ds, acc.a / total));
    if (mix_t <= 0.0) {
        textureStore(dst, local, sharp);
        return;
    }
    let out = sharp.rgb + (acc.rgb / total - sharp.rgb) * mix_t;
    textureStore(dst, local, vec4<f32>(out, sharp.a));
}
