use std::sync::Arc;

use half::f16;
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use super::GpuRenderer;
use crate::gpu::source::LinearSource;
use crate::source::SourceImage;
use crate::{PipelineError, PipelineResult};

impl GpuRenderer {
    pub fn upload_source(&self, image: &SourceImage) -> PipelineResult<LinearSource> {
        let (w, h) = image.header.dims;
        let max_dim = self.ctx.device.limits().max_texture_dimension_2d;
        if w == 0 || h == 0 || w > max_dim || h > max_dim {
            return Err(PipelineError::Unsupported(format!(
                "source dims {w}x{h} outside 1..={max_dim}"
            )));
        }
        if image.rgb_f16.len() != w as usize * h as usize * 3 {
            return Err(PipelineError::Render(format!(
                "source holds {} samples, expected {w}x{h}x3",
                image.rgb_f16.len()
            )));
        }
        let format = self.ctx.linear_format;
        let texels = texel_bytes(&image.rgb_f16, format);
        let texture = self.ctx.device.create_texture(&TextureDescriptor {
            label: Some("linear-source-upload"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.ctx.queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * texel_size(format)),
                rows_per_image: Some(h),
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        Ok(LinearSource {
            meta: image.header.meta.clone(),
            kind: image.header.kind,
            dims: (w, h),
            texture: Arc::new(texture),
            atmosphere: image.header.atmosphere,
        })
    }

    #[cfg(feature = "native")]
    pub fn read_source(
        &self,
        source: &LinearSource,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<SourceImage> {
        let (w, h) = source.dims;
        let format = source.texture.format();
        let bpp = texel_size(format);
        let padded = (w * bpp).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let device = &self.ctx.device;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("linear-source-readback"),
            size: padded as u64 * h as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("linear-source-readback-enc"),
        });
        encoder.copy_texture_to_buffer(
            source.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.ctx.queue.submit(Some(encoder.finish()));
        crate::gpu::readback::map_buffer_cancellable(&self.ctx, &buffer, cancel)?;
        let slice = buffer.slice(..);
        let data = crate::gpu::readback::mapped_range(&slice)?;
        let row_bytes = (w * bpp) as usize;
        let mut rgb_f16 = Vec::with_capacity(w as usize * h as usize * 3);
        for row in data.chunks_exact(padded as usize) {
            extend_rgb(&mut rgb_f16, &row[..row_bytes], format);
        }
        drop(data);
        buffer.unmap();
        Ok(SourceImage {
            header: source.header(),
            rgb_f16,
        })
    }
}

fn texel_size(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::Rgba32Float => 16,
        _ => 8,
    }
}

fn texel_bytes(rgb_f16: &[u16], format: TextureFormat) -> Vec<u8> {
    let one = f16::ONE.to_bits();
    match format {
        TextureFormat::Rgba32Float => rgb_f16
            .chunks_exact(3)
            .flat_map(|px| {
                [px[0], px[1], px[2], one]
                    .map(|bits| f16::from_bits(bits).to_f32())
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
            })
            .collect(),
        _ => rgb_f16
            .chunks_exact(3)
            .flat_map(|px| {
                [px[0], px[1], px[2], one]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
            })
            .collect(),
    }
}

#[cfg(feature = "native")]
fn extend_rgb(out: &mut Vec<u16>, row: &[u8], format: TextureFormat) {
    match format {
        TextureFormat::Rgba32Float => out.extend(row.chunks_exact(16).flat_map(|px| {
            let channel = |i: usize| {
                let bytes = [px[i * 4], px[i * 4 + 1], px[i * 4 + 2], px[i * 4 + 3]];
                f16::from_f32(f32::from_le_bytes(bytes)).to_bits()
            };
            [channel(0), channel(1), channel(2)]
        })),
        _ => out.extend(row.chunks_exact(8).flat_map(|px| {
            [
                u16::from_le_bytes([px[0], px[1]]),
                u16::from_le_bytes([px[2], px[3]]),
                u16::from_le_bytes([px[4], px[5]]),
            ]
        })),
    }
}
