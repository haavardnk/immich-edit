@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.grid_size.x || gid.y >= p.grid_size.y) {
        return;
    }
    let od = oriented_dims();
    let cell = p.grid_origin + gid.xy;
    var acc = vec4<f32>(0.0);
    var count = 0.0;
    for (var j = 0u; j < p.ds; j = j + 1u) {
        let oy = cell.y * p.ds + j;
        if (oy >= od.y) {
            break;
        }
        for (var i = 0u; i < p.ds; i = i + 1u) {
            let ox = cell.x * p.ds + i;
            if (ox >= od.x) {
                break;
            }
            let s = oriented_to_sensor(vec2<u32>(ox, oy));
            if (s.x < p.origin.x || s.y < p.origin.y) {
                continue;
            }
            let local = s - p.origin;
            if (local.x >= p.size.x || local.y >= p.size.y) {
                continue;
            }
            let rgb = textureLoad(src, vec2<i32>(local), 0).rgb;
            acc = acc + vec4<f32>(rgb, signed_coc_at(vec2<u32>(ox, oy)));
            count = count + 1.0;
        }
    }
    let out = vec4<f32>(acc.rgb / count, acc.a / count / f32(p.ds));
    textureStore(dst, vec2<i32>(gid.xy), out);
}
