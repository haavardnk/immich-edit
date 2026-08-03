// color-space: linear scene-referred Rgba16Float in/out; composites one stroke into the image
struct Params {
    img: vec2<u32>,
    bbox_origin: vec2<u32>,
    bbox_size: vec2<u32>,
    n_points: u32,
    mode: u32,
    offset: vec2<f32>,
    radius: f32,
    hardness: f32,
    opacity: f32,
    sigma: f32,
    dir: u32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var patch_src: texture_2d<f32>;
@group(0) @binding(3) var patch_res: texture_2d<f32>;
@group(0) @binding(4) var<storage, read> pts: array<vec2<f32>>;
@group(0) @binding(5) var out_tex: texture_storage_2d<rgba16float, write>;

fn point_seg_dist(pt: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = b - a;
    let len2 = dot(d, d);
    var t = 0.0;
    if (len2 > 1e-12) {
        t = clamp(dot(pt - a, d) / len2, 0.0, 1.0);
    }
    return length(pt - (a + t * d));
}

fn coverage(dist: f32) -> f32 {
    if (dist >= p.radius) { return 0.0; }
    let inner = p.radius * clamp(p.hardness, 0.0, 1.0);
    if (dist <= inner) { return 1.0; }
    let falloff = p.radius - inner;
    if (falloff <= 1e-6) { return 1.0; }
    let t = (p.radius - dist) / falloff;
    return t * t * (3.0 - 2.0 * t);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.img.x || gid.y >= p.img.y) { return; }
    let c = vec2<i32>(i32(gid.x), i32(gid.y));
    var col = textureLoad(src_tex, c, 0).rgb;
    let inside = gid.x >= p.bbox_origin.x && gid.x < p.bbox_origin.x + p.bbox_size.x
        && gid.y >= p.bbox_origin.y && gid.y < p.bbox_origin.y + p.bbox_size.y;
    if (inside) {
        let pt = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
        var best = 3.4e38;
        if (p.n_points == 1u) {
            best = length(pt - pts[0]);
        } else {
            for (var i = 0u; i + 1u < p.n_points; i = i + 1u) {
                best = min(best, point_seg_dist(pt, pts[i], pts[i + 1u]));
            }
        }
        let cov = coverage(best) * p.opacity;
        if (cov > 0.0) {
            let lc = vec2<i32>(i32(gid.x - p.bbox_origin.x), i32(gid.y - p.bbox_origin.y));
            var s = textureLoad(patch_src, lc, 0).rgb;
            if (p.mode == 0u) {
                s = max(s + textureLoad(patch_res, lc, 0).rgb, vec3<f32>(0.0));
            }
            col = mix(col, s, cov);
        }
    }
    textureStore(out_tex, c, vec4<f32>(col, 1.0));
}
