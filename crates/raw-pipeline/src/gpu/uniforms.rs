use std::mem::offset_of;

pub(super) const ACTIVE_MASK_WORDS: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ProcessHeader {
    pub src_size: [u32; 2],
    pub out_size: [u32; 2],
    pub crop: [f32; 4],
    pub flags: [u32; 4],
    pub geom_extra: [f32; 4],
    pub active_mask: [u32; ACTIVE_MASK_WORDS],
    pub geom_extra2: [f32; 4],
    pub geom_extra3: [f32; 4],
    pub output: [u32; 4],
    pub perspective: [f32; 12],
}

pub(super) fn write_header(dst: &mut [u8], header: &ProcessHeader) {
    dst[..size_of::<ProcessHeader>()].copy_from_slice(bytemuck::bytes_of(header));
}

pub(super) fn write_active_mask(dst: &mut [u8], mask: [u32; ACTIVE_MASK_WORDS]) {
    let off = offset_of!(ProcessHeader, active_mask);
    dst[off..off + size_of_val(&mask)].copy_from_slice(bytemuck::cast_slice(&mask));
}
