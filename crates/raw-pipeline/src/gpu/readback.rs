use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, Extent3d, ImageCopyBuffer,
    ImageDataLayout, MapMode, Origin3d, Texture, TextureAspect,
};

use super::context::GpuContext;
use crate::{PipelineError, PipelineResult};

const ROW_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

fn padded_row(width: u32, bpp: u32) -> u32 {
    let unpadded = width * bpp;
    let rem = unpadded % ROW_ALIGN;
    if rem == 0 {
        unpadded
    } else {
        unpadded + (ROW_ALIGN - rem)
    }
}

pub fn padded_row_bytes(width: u32) -> u32 {
    padded_row(width, 4)
}

fn padded_row_bytes_f16(width: u32) -> u32 {
    padded_row(width, 8)
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

pub fn make_readback_buffer_f16(device: &Device, width: u32, height: u32) -> Buffer {
    let size = (padded_row_bytes_f16(width) as u64) * (height as u64);
    device.create_buffer(&BufferDescriptor {
        label: Some("readback-linear"),
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
    let bpp: u32 = match texture.format() {
        wgpu::TextureFormat::Rgba16Float => 8,
        _ => 4,
    };
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
                bytes_per_row: Some(padded_row(width, bpp)),
                rows_per_image: Some(height),
            },
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

pub fn read_rgba8(
    ctx: &GpuContext,
    buffer: &Buffer,
    width: u32,
    height: u32,
) -> PipelineResult<Vec<u8>> {
    let padded = padded_row_bytes(width) as usize;
    let unpadded = (width * 4) as usize;
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

pub fn read_rgba16f_as_rgb(
    ctx: &GpuContext,
    buffer: &Buffer,
    width: u32,
    height: u32,
) -> PipelineResult<Vec<f32>> {
    let padded = padded_row_bytes_f16(width) as usize;
    let unpadded_bytes = (width * 8) as usize;
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
    let px_count = (width * height) as usize;
    let mut out = Vec::with_capacity(px_count * 3);
    for row in 0..height as usize {
        let start = row * padded;
        let row_u16: &[u16] = bytemuck::cast_slice(&data[start..start + unpadded_bytes]);
        for px in row_u16.chunks_exact(4) {
            out.push(half::f16::from_bits(px[0]).to_f32());
            out.push(half::f16::from_bits(px[1]).to_f32());
            out.push(half::f16::from_bits(px[2]).to_f32());
        }
    }
    drop(data);
    buffer.unmap();
    Ok(out)
}
