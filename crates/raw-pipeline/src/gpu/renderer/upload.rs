use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::frame::RawFrame;
use crate::gpu::helpers::{DemosaicParams, cfa_to_indices, mip_count};
use crate::{PipelineError, PipelineResult};

use super::{CachedFrame, GpuRenderer};

impl GpuRenderer {
    pub(super) fn frame_key(frame: &RawFrame) -> u64 {
        let ptr = frame.data.as_ptr() as usize as u64;
        let dims = ((frame.width as u64) << 32) | (frame.height as u64);
        ptr ^ dims
    }

    pub(super) fn get_or_demosaic(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let key = Self::frame_key(frame);
        if let Some(c) = self.cache.lock().get(&key).cloned() {
            return Ok(c);
        }
        let cached = if frame.cpp == 3 {
            self.upload_rgb_texture(frame)?
        } else {
            self.demosaic_to_texture(frame)?
        };
        self.cache.lock().put(key, cached.clone());
        Ok(cached)
    }

    fn upload_rgb_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.upload_rgb",
            w = frame.width as u32,
            h = frame.height as u32
        )
        .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.width as u32;
        let h = frame.height as u32;

        let rgba_f16: Vec<u16> = frame
            .data
            .chunks_exact(3)
            .flat_map(|rgb| {
                [
                    half::f16::from_f32(rgb[0]).to_bits(),
                    half::f16::from_f32(rgb[1]).to_bits(),
                    half::f16::from_f32(rgb[2]).to_bits(),
                    half::f16::from_f32(1.0).to_bits(),
                ]
            })
            .collect();

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-uploaded"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&rgba_f16),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 8),
                rows_per_image: Some(h),
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("upload-mipgen-enc"),
        });
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn demosaic_to_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.demosaic",
            w = frame.width as u32,
            h = frame.height as u32
        )
        .entered();
        if frame.cpp != 1 {
            return Err(PipelineError::Unsupported(
                "gpu demosaic requires single-plane bayer frame".into(),
            ));
        }
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.width as u32;
        let h = frame.height as u32;

        let cfa = cfa_to_indices(&frame.cfa_pattern);
        let params = DemosaicParams {
            size: [w, h],
            _pad: [0, 0],
            cfa,
        };

        let uniform_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&params),
            "demosaic-uniform",
        );

        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-cached"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("demosaic-bg"),
            layout: &self.passes.demosaic.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: raw_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("demosaic-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("demosaic-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.demosaic.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            let gx = w.div_ceil(16);
            let gy = h.div_ceil(16);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    pub(super) fn encode_mipgen(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &Texture,
        w: u32,
        h: u32,
    ) {
        let _span = tracing::trace_span!("gpu.mipgen", w = w, h = h).entered();
        let levels = mip_count(w, h);
        if levels <= 1 {
            return;
        }
        let device = &self.ctx.device;
        let mut mip_w = w;
        let mut mip_h = h;
        for level in 1..levels {
            let src_view = texture.create_view(&TextureViewDescriptor {
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_w = (mip_w / 2).max(1);
            let dst_h = (mip_h / 2).max(1);
            let dst_view = texture.create_view(&TextureViewDescriptor {
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("mipgen-bg"),
                layout: &self.passes.mipgen.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("mipgen-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.passes.mipgen.pipeline);
                pass.set_bind_group(0, &bind, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
            }
            mip_w = dst_w;
            mip_h = dst_h;
        }
    }
}
