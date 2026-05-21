use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DemosaicParams {
    pub size: [u32; 2],
    pub _pad: [u32; 2],
    pub cfa: [u32; 4],
    pub black: [f32; 4],
    pub inv_range: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ProcessParams {
    pub src_size: [u32; 2],
    pub out_size: [u32; 2],
    pub crop: [f32; 4],
    pub wb: [f32; 4],
    pub tone: [f32; 4],
    pub flags: [u32; 4],
    pub sat: f32,
    pub _pad: [f32; 3],
}
