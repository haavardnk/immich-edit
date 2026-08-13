use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    CommandEncoder, ComputePassDescriptor, ComputePipeline, Device, Sampler, TextureView,
};

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

pub(super) fn dispatch_2d(
    encoder: &mut CommandEncoder,
    label: &str,
    pipeline: &ComputePipeline,
    bind: &BindGroup,
    gx: u32,
    gy: u32,
) {
    let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    });
    cp.set_pipeline(pipeline);
    cp.set_bind_group(0, bind, &[]);
    cp.dispatch_workgroups(gx, gy, 1);
}
