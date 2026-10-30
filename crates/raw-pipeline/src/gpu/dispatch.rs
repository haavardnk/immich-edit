use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    CommandEncoder, ComputePass, ComputePassDescriptor, ComputePipeline, Device, Origin3d, Sampler,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureView,
};

use super::texture::extent_2d;
use super::timer::with_pass_timestamps;

pub(super) fn tex(view: &TextureView) -> BindingResource<'_> {
    BindingResource::TextureView(view)
}

pub(super) fn buf(buffer: &Buffer) -> BindingResource<'_> {
    buffer.as_entire_binding()
}

pub(super) fn samp(sampler: &Sampler) -> BindingResource<'_> {
    BindingResource::Sampler(sampler)
}

pub(super) fn bind_group(
    device: &Device,
    label: &str,
    layout: &BindGroupLayout,
    resources: &[BindingResource<'_>],
) -> BindGroup {
    let entries: Vec<BindGroupEntry> = resources
        .iter()
        .enumerate()
        .map(|(i, resource)| BindGroupEntry {
            binding: i as u32,
            resource: resource.clone(),
        })
        .collect();
    bind_group_indexed(device, label, layout, &entries)
}

pub(super) fn bind_group_indexed(
    device: &Device,
    label: &str,
    layout: &BindGroupLayout,
    entries: &[BindGroupEntry<'_>],
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries,
    })
}

pub(super) fn begin_pass<'e>(encoder: &'e mut CommandEncoder, label: &str) -> ComputePass<'e> {
    with_pass_timestamps(|timestamp_writes| {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some(label),
            timestamp_writes,
        })
    })
}

pub(super) fn dispatch_2d(
    encoder: &mut CommandEncoder,
    label: &str,
    pipeline: &ComputePipeline,
    bind: &BindGroup,
    gx: u32,
    gy: u32,
) {
    let mut cp = begin_pass(encoder, label);
    cp.set_pipeline(pipeline);
    cp.set_bind_group(0, bind, &[]);
    cp.dispatch_workgroups(gx, gy, 1);
}

pub(super) fn copy_texture(
    encoder: &mut CommandEncoder,
    src: &Texture,
    dst: &Texture,
    dims: (u32, u32),
) {
    copy_region(
        encoder,
        Region::at(src, 0, (0, 0)),
        Region::at(dst, 0, (0, 0)),
        dims,
    );
}

#[derive(Clone, Copy)]
pub(super) struct Region<'t> {
    texture: &'t Texture,
    level: u32,
    origin: (u32, u32),
}

impl<'t> Region<'t> {
    pub fn at(texture: &'t Texture, level: u32, origin: (u32, u32)) -> Self {
        Self {
            texture,
            level,
            origin,
        }
    }

    fn info(self) -> TexelCopyTextureInfo<'t> {
        TexelCopyTextureInfo {
            texture: self.texture,
            mip_level: self.level,
            origin: Origin3d {
                x: self.origin.0,
                y: self.origin.1,
                z: 0,
            },
            aspect: TextureAspect::All,
        }
    }
}

pub(super) fn copy_region(
    encoder: &mut CommandEncoder,
    src: Region<'_>,
    dst: Region<'_>,
    dims: (u32, u32),
) {
    encoder.copy_texture_to_texture(src.info(), dst.info(), extent_2d(dims));
}
