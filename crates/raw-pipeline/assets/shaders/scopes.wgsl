// color-space: display-referred 8-bit levels in, waveform/parade/vectorscope counts out

struct BinParams {
    size: vec2<u32>,
    step: u32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: BinParams;
// DISPLAY_LOAD_INJECT
@group(0) @binding(2) var<storage, read_write> cells: array<atomic<u32>>;

const LEVELS: u32 = 256u;
const WAVEFORM_COLUMNS: u32 = 512u;
const PARADE_COLUMNS: u32 = 192u;
const PARADE_BASE: u32 = 131072u;
const VECTORSCOPE_BASE: u32 = 278528u;
const VECTORSCOPE_SIZE: i32 = 384;
const LUMA_SCALE: i32 = 10000;
const CB_DENOM: i32 = 4731780;
const CR_DENOM: i32 = 4015740;

fn parade_cell(level: u32, column: u32, channel: u32) -> u32 {
    return PARADE_BASE + ((LEVELS - 1u - level) * PARADE_COLUMNS + column) * 3u + channel;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y || gid.y % p.step != 0u) {
        return;
    }
    let d = load_display_u8(vec2<i32>(gid.xy));
    let weighted = 2126u * d.r + 7152u * d.g + 722u * d.b;
    let level = weighted / 10000u;
    atomicAdd(&cells[(LEVELS - 1u - level) * WAVEFORM_COLUMNS + gid.x * WAVEFORM_COLUMNS / p.size.x], 1u);

    let column = gid.x * PARADE_COLUMNS / p.size.x;
    atomicAdd(&cells[parade_cell(d.r, column, 0u)], 1u);
    atomicAdd(&cells[parade_cell(d.g, column, 1u)], 1u);
    atomicAdd(&cells[parade_cell(d.b, column, 2u)], 1u);

    let y = i32(weighted);
    let half = VECTORSCOPE_SIZE / 2;
    let vx = (VECTORSCOPE_SIZE * (LUMA_SCALE * i32(d.b) - y) + half * CB_DENOM) / CB_DENOM;
    let vy = (half * CR_DENOM - VECTORSCOPE_SIZE * (LUMA_SCALE * i32(d.r) - y)) / CR_DENOM;
    let cx = u32(clamp(vx, 0, VECTORSCOPE_SIZE - 1));
    let cy = u32(clamp(vy, 0, VECTORSCOPE_SIZE - 1));
    atomicAdd(&cells[VECTORSCOPE_BASE + cy * u32(VECTORSCOPE_SIZE) + cx], 1u);
}
