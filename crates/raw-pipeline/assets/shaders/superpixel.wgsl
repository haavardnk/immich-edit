struct SuperpixelParams {
    size: vec2<u32>,
    block: u32,
    period: u32,
    pattern: array<vec4<u32>, 9>,
};

@group(0) @binding(0) var<uniform> p: SuperpixelParams;
@group(0) @binding(1) var<storage, read> raw_in: array<f32>;
@group(0) @binding(2) var rgb_out: texture_storage_2d<rgba16float, write>;

fn channel_at(x: u32, y: u32) -> u32 {
    let i = (y % p.period) * p.period + (x % p.period);
    return p.pattern[i / 4u][i % 4u];
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let out_size = p.size / p.block;
    if (gid.x >= out_size.x || gid.y >= out_size.y) {
        return;
    }
    var sum = vec3<f32>(0.0);
    var count = vec3<f32>(0.0);
    for (var i = 0u; i < p.block * p.block; i = i + 1u) {
        let x = gid.x * p.block + i % p.block;
        let y = gid.y * p.block + i / p.block;
        let ch = channel_at(x, y);
        sum[ch] = sum[ch] + raw_in[y * p.size.x + x];
        count[ch] = count[ch] + 1.0;
    }
    let rgb = sum / max(count, vec3<f32>(1.0));
    textureStore(rgb_out, vec2<i32>(gid.xy), vec4<f32>(rgb, 1.0));
}
