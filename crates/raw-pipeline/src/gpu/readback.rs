use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, Extent3d, ImageCopyBuffer,
    ImageDataLayout, MapMode, Origin3d, Texture, TextureAspect,
};

use crate::{PipelineError, PipelineResult};
use super::context::GpuContext;

const BYTES_PER_PIXEL: u32 = 4;
const ROW_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub fn padded_row_bytes(width: u32) -> u32 {
    let unpadded = width * BYTES_PER_PIXEL;
    let rem = unpadded % ROW_ALIGN;
    if rem == 0 { unpadded } else { unpadded + (ROW_ALIGN - rem) }
}

pub fn make_readback_buffer(device: &Device, width: u32, height: u32) -> Buffer {
    let size = (padded_row_bytes(width) as u64) * (height as u64);
    device.create_buffer(&BufferDescriptor {
        label: Some("readback"),
        size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

pub fn copy_texture_to_buffer(
    encoder: &mut CommandEncoder,
    texture: &Texture,
    buffer: &Buffer,
    width: u32,
    height: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        ImageCopyBuffer {
            buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes(width)),
                rows_per_image: Some(height),
            },
        },
        Extent3d { width, height, depth_or_array_layers: 1 },
    );
}

pub fn read_rgba8(
    ctx: &GpuContext,
    buffer: &Buffer,
    width: u32,
    height: u32,
) -> PipelineResult<Vec<u8>> {
    let padded = padded_row_bytes(width) as usize;
    let unpadded = (width * BYTES_PER_PIXEL) as usize;
    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |r| {
        let _ = sender.send(r);
    });
    ctx.device.poll(wgpu::Maintain::Wait);
    receiver
        .recv()
        .map_err(|e| PipelineError::Render(format!("readback recv: {e}")))?
        .map_err(|e| PipelineError::Render(format!("readback map: {e}")))?;

    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity(unpadded * height as usize);
    for row in 0..height as usize {
        let start = row * padded;
        out.extend_from_slice(&data[start..start + unpadded]);
    }
    drop(data);
    buffer.unmap();
    Ok(out)
}
