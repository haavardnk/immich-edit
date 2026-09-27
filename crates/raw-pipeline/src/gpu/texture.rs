use wgpu::{
    Device, Extent3d, Queue, TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

pub(super) const STORAGE_SAMPLED: TextureUsages =
    TextureUsages::STORAGE_BINDING.union(TextureUsages::TEXTURE_BINDING);

pub(super) fn extent_2d((width, height): (u32, u32)) -> Extent3d {
    Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}

pub(super) fn texture_2d(
    device: &Device,
    label: &str,
    format: TextureFormat,
    dims: (u32, u32),
    mip_level_count: u32,
    usage: TextureUsages,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(label),
        size: extent_2d(dims),
        mip_level_count,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub(super) fn write_texture_2d(
    queue: &Queue,
    texture: &Texture,
    bytes: &[u8],
    bytes_per_row: u32,
    (width, height): (u32, u32),
) {
    queue.write_texture(
        texture.as_image_copy(),
        bytes,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(height),
        },
        extent_2d((width, height)),
    );
}

pub(super) fn full_view(texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor::default())
}

pub(super) fn mip_view(texture: &Texture, level: u32) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        base_mip_level: level,
        mip_level_count: Some(1),
        ..Default::default()
    })
}
