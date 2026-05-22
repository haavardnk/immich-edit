use crate::gpu::shader_builder::ACTIVE_MASK_OFFSET;

pub(super) const ACTIVE_MASK_WORDS: usize = 4;

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

pub(super) fn write_active_mask(dst: &mut [u8], mask: [u32; ACTIVE_MASK_WORDS]) {
    dst[ACTIVE_MASK_OFFSET..ACTIVE_MASK_OFFSET + 16].copy_from_slice(bytemuck::cast_slice(&mask));
}
