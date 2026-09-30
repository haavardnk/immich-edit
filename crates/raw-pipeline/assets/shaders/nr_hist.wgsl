struct NrHistParams {
    size: vec2<u32>,
    lo_size: vec2<u32>,
    lo: u32,
    count: u32,
    offset: u32,
    fine: u32,
}

@group(0) @binding(0) var<uniform> p: NrHistParams;
@group(0) @binding(1) var fine_src: texture_2d<f32>;
@group(0) @binding(2) var smooth_src: texture_2d<f32>;
@group(0) @binding(3) var reference_src: texture_2d<f32>;
@group(0) @binding(4) var<storage, read_write> hist: array<atomic<u32>>;

const THREADS: u32 = 256u;
const SIDE: u32 = HIST_TILE / SAMPLE_STRIDE;
const LOCAL_BINS: u32 = 2u * HIST_LEN;

var<workgroup> local_bins: array<atomic<u32>, LOCAL_BINS>;

fn sample_detail(q: vec2<u32>) -> vec4<f32> {
    let c = vec2<i32>(q);
    if (p.fine == 1u) {
        let d = to_chroma(textureLoad(fine_src, c, 0).rgb) - upsample(smooth_src, q, p.lo_size).rgb;
        return vec4<f32>(0.0, d.y, d.z, upsample(reference_src, q, p.lo_size).r);
    }
    let next = textureLoad(smooth_src, c, 0);
    return vec4<f32>((textureLoad(fine_src, c, 0) - next).rgb, next.r);
}

@compute @workgroup_size(16, 16, 1)
fn main(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let bins = p.count * HIST_LEN;
    for (var i = li; i < bins; i += THREADS) {
        atomicStore(&local_bins[i], 0u);
    }
    workgroupBarrier();
    let origin = wg.xy * HIST_TILE;
    for (var k = li; k < SIDE * SIDE; k += THREADS) {
        let q = origin + vec2<u32>(k % SIDE, k / SIDE) * SAMPLE_STRIDE;
        if (q.x >= p.size.x || q.y >= p.size.y) {
            continue;
        }
        let s = sample_detail(q);
        for (var ch = 0u; ch < p.count; ch += 1u) {
            let idx = hist_index(s[p.lo + ch], s.a);
            if (idx >= 0) {
                atomicAdd(&local_bins[ch * HIST_LEN + u32(idx)], 1u);
            }
        }
    }
    workgroupBarrier();
    for (var i = li; i < bins; i += THREADS) {
        let v = atomicLoad(&local_bins[i]);
        if (v != 0u) {
            atomicAdd(&hist[p.offset + i], v);
        }
    }
}
