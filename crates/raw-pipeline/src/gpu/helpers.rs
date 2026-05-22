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

pub(super) fn scale_to_max(w: u32, h: u32, max_edge: u32) -> (u32, u32) {
    if w <= max_edge && h <= max_edge {
        return (w, h);
    }
    let scale = max_edge as f64 / w.max(h) as f64;
    let nw = ((w as f64) * scale).round() as u32;
    let nh = ((h as f64) * scale).round() as u32;
    (nw.max(1), nh.max(1))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_header(
    dst: &mut [u8],
    src_size: [u32; 2],
    out_size: [u32; 2],
    crop: [f32; 4],
    flags: [u32; 4],
    geom_extra: [f32; 4],
    geom_extra2: [f32; 4],
    geom_extra3: [f32; 4],
) {
    dst[0..8].copy_from_slice(bytemuck::cast_slice(&src_size));
    dst[8..16].copy_from_slice(bytemuck::cast_slice(&out_size));
    dst[16..32].copy_from_slice(bytemuck::cast_slice(&crop));
    dst[32..48].copy_from_slice(bytemuck::cast_slice(&flags));
    dst[48..64].copy_from_slice(bytemuck::cast_slice(&geom_extra));
    dst[80..96].copy_from_slice(bytemuck::cast_slice(&geom_extra2));
    dst[96..112].copy_from_slice(bytemuck::cast_slice(&geom_extra3));
}
