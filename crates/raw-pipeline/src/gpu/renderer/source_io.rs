use std::sync::Arc;

use half::f16;
use wgpu::{TextureFormat, TextureUsages};

use super::GpuRenderer;
use crate::gpu::source::LinearSource;
#[cfg(feature = "native")]
use crate::gpu::texture::extent_2d;
use crate::gpu::texture::{texture_2d, write_texture_2d};
use crate::source::SourceImage;
#[cfg(feature = "native")]
use crate::source::{SourceHeader, SourceWindow, WindowRect};
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
        let texture = texture_2d(
            &self.ctx.device,
            "linear-source-upload",
            format,
            (w, h),
            1,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC,
        );
        write_texture_2d(
            &self.ctx.queue,
            &texture,
            &texels,
            w * texel_size(format),
            (w, h),
        );
        Ok(LinearSource {
            meta: image.header.meta.clone(),
            kind: image.header.kind,
            dims: (w, h),
            texture: Arc::new(texture),
            atmosphere: image.header.atmosphere,
            window: image.header.window,
        })
    }

    #[cfg(feature = "native")]
    pub fn read_source(
        &self,
        source: &LinearSource,
        window: Option<WindowRect>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<SourceImage> {
        let (origin, (w, h)) = match window {
            Some(rect) => (rect.origin, rect.dims),
            None => ((0, 0), source.dims),
        };
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
            wgpu::TexelCopyTextureInfo {
                texture: &source.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: origin.0,
                    y: origin.1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            extent_2d((w, h)),
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
        let header = SourceHeader {
            dims: (w, h),
            window: window.map(|rect| SourceWindow {
                origin: rect.origin,
                full: source.dims,
            }),
            ..source.header()
        };
        Ok(SourceImage { header, rgb_f16 })
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
    let texel = texel_size(format) as usize;
    let mut out = vec![0u8; rgb_f16.len() / 3 * texel];
    let pixels = rgb_f16.chunks_exact(3).zip(out.chunks_exact_mut(texel));
    match format {
        TextureFormat::Rgba32Float => pixels.for_each(|(px, dst)| {
            for (bits, d) in [px[0], px[1], px[2], one]
                .into_iter()
                .zip(dst.chunks_exact_mut(4))
            {
                d.copy_from_slice(&f16::from_bits(bits).to_f32().to_le_bytes());
            }
        }),
        _ => pixels.for_each(|(px, dst)| {
            for (bits, d) in [px[0], px[1], px[2], one]
                .into_iter()
                .zip(dst.chunks_exact_mut(2))
            {
                d.copy_from_slice(&bits.to_le_bytes());
            }
        }),
    }
    out
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

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn texels_round_trip_through_readback() {
        let (w, h) = (5usize, 3usize);
        let rgb: Vec<u16> = (0..w * h * 3).map(|i| (i * 997 % 0x7c00) as u16).collect();
        for (format, alpha) in [
            (
                TextureFormat::Rgba16Float,
                f16::ONE.to_bits().to_le_bytes().to_vec(),
            ),
            (TextureFormat::Rgba32Float, 1f32.to_le_bytes().to_vec()),
        ] {
            let texels = texel_bytes(&rgb, format);
            let texel = texel_size(format) as usize;
            let mut back = Vec::new();
            for row in texels.chunks_exact(w * texel) {
                extend_rgb(&mut back, row, format);
            }
            assert_eq!(back, rgb, "{format:?}");
            assert!(
                texels
                    .chunks_exact(texel)
                    .all(|px| px[texel * 3 / 4..] == alpha[..]),
                "{format:?}"
            );
        }
    }
}
