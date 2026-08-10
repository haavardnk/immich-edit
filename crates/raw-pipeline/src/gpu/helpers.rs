#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct DemosaicParams {
    pub size: [u32; 2],
    pub _pad: [u32; 2],
    pub cfa: [u32; 4],
}

pub(super) fn round_up_256(v: u32) -> u32 {
    (v + 255) & !255
}

pub(super) fn mip_count(w: u32, h: u32) -> u32 {
    (w.max(h) as f32).log2().floor() as u32 + 1
}

pub(super) fn cfa_to_indices(pattern: &str) -> [u32; 4] {
    let mut out = [1u32; 4];
    for (i, c) in pattern.chars().take(4).enumerate() {
        out[i] = match c {
            'R' => 0,
            'G' => 1,
            'B' => 2,
            _ => 1,
        };
    }
    out
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct XtransParams {
    pub size: [u32; 2],
    pub _pad: [u32; 2],
    pub pattern: [[u32; 4]; 9],
}

pub(super) fn xtrans_to_indices(pattern: &[u8; 36]) -> [[u32; 4]; 9] {
    let mut out = [[1u32; 4]; 9];
    for (i, b) in pattern.iter().enumerate() {
        out[i / 4][i % 4] = match b {
            b'R' => 0,
            b'B' => 2,
            _ => 1,
        };
    }
    out
}
