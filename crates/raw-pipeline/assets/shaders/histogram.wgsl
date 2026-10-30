// color-space: display-referred 8-bit levels and scene-linear output in, 256-bin counts out

struct BinParams {
    core: vec4<u32>,
    origin: vec2<u32>,
    full: vec2<u32>,
    step: u32,
};

@group(0) @binding(0) var<uniform> p: BinParams;
// DISPLAY_LOAD_INJECT
@group(0) @binding(2) var linear_tex: texture_2d<f32>;
@group(0) @binding(3) var<storage, read_write> bins: array<atomic<u32>, 2048>;

const TILE: u32 = 64u;
const THREADS: u32 = 256u;
const TOTAL_BINS: u32 = 2048u;
const DISPLAY_BASE: u32 = 0u;
const LINEAR_BASE: u32 = 1024u;

var<workgroup> local_bins: array<atomic<u32>, 2048>;

fn display_luma(c: vec3<u32>) -> u32 {
    return (2126u * c.r + 7152u * c.g + 722u * c.b) / 10000u;
}

fn linear_bin(v: f32) -> u32 {
    return min(u32(clamp(v, 0.0, 1.0) * 255.0), 255u);
}

fn count(base: u32, r: u32, g: u32, b: u32, l: u32) {
    atomicAdd(&local_bins[base + r], 1u);
    atomicAdd(&local_bins[base + 256u + g], 1u);
    atomicAdd(&local_bins[base + 512u + b], 1u);
    atomicAdd(&local_bins[base + 768u + l], 1u);
}

@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    for (var i = li; i < TOTAL_BINS; i += THREADS) {
        atomicStore(&local_bins[i], 0u);
    }
    workgroupBarrier();
    let start = wg.xy * TILE;
    for (var k = li; k < TILE * TILE; k += THREADS) {
        let local = start + vec2<u32>(k % TILE, k / TILE);
        if (local.x >= p.core.z || local.y >= p.core.w) {
            continue;
        }
        let x = p.core.x + local.x;
        let y = p.core.y + local.y;
        if ((y * p.full.x + x) % p.step != 0u) {
            continue;
        }
        let c = vec2<i32>(vec2<u32>(x, y) - p.origin);
        let d = load_display_u8(c);
        count(DISPLAY_BASE, d.r, d.g, d.b, display_luma(d));
        let l = textureLoad(linear_tex, c, 0).rgb;
        let luma = 0.2126 * l.r + 0.7152 * l.g + 0.0722 * l.b;
        count(LINEAR_BASE, linear_bin(l.r), linear_bin(l.g), linear_bin(l.b), linear_bin(luma));
    }
    workgroupBarrier();
    for (var i = li; i < TOTAL_BINS; i += THREADS) {
        let v = atomicLoad(&local_bins[i]);
        if (v != 0u) {
            atomicAdd(&bins[i], v);
        }
    }
}
