use std::hash::{Hash, Hasher};
use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoderDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::helpers::{
    DemosaicParams, SuperpixelParams, XtransParams, cfa_to_indices, mip_count, xtrans_to_indices,
};
use crate::{PipelineError, PipelineResult};

use super::{CachedFrame, GpuRenderer};

impl GpuRenderer {
    pub(super) fn get_or_demosaic(
        &self,
        frame: &RawFrame,
        block: Option<usize>,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let cache = match block {
            Some(_) => &self.sensor.superpixels,
            None => &self.sensor.frames,
        };
        let key = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            frame.cache_key().hash(&mut h);
            block.hash(&mut h);
            h.finish()
        };
        if let Some(c) = cache.lock().get(&key).cloned() {
            return Ok(c);
        }
        let cached = match block {
            Some(block) => self.superpixel_to_texture(frame, block)?,
            None if frame.cpp == 3 => self.upload_rgb_texture(frame)?,
            None => self.demosaic_to_texture(frame)?,
        };
        cache.lock().put(key, cached.clone());
        Ok(cached)
    }

    fn superpixel_to_texture(
        &self,
        frame: &RawFrame,
        block: usize,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (period, pattern) = match crate::cpu::demosaic::parse_xtrans(&frame.cfa_pattern) {
            Some(pattern) => (6, xtrans_to_indices(&pattern)),
            None => {
                let cfa = cfa_to_indices(&frame.cfa_pattern);
                let mut pattern = [[1u32; 4]; 9];
                pattern[0] = cfa;
                (2, pattern)
            }
        };
        let params = SuperpixelParams {
            size: [frame.meta.width as u32, frame.meta.height as u32],
            block: block as u32,
            period,
            pattern,
        };
        let w = (frame.meta.width / block) as u32;
        let h = (frame.meta.height / block) as u32;
        let uniform_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&params),
            "superpixel-uniform",
        );
        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("superpixel-raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-superpixel"),
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
        let bind = bind_group(
            device,
            "superpixel-bg",
            &self.passes.demosaic.layout,
            &[uniform_buf.as_entire_binding(), buf(&raw_buf), tex(&view)],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("superpixel-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "superpixel-pass",
            &self.passes.demosaic.superpixel,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));
        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
            block,
        }))
    }

    fn upload_rgb_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.upload_rgb",
            w = frame.meta.width as u32,
            h = frame.meta.height as u32
        )
        .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.meta.width as u32;
        let h = frame.meta.height as u32;

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
            block: 1,
        }))
    }

    fn demosaic_to_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.demosaic",
            w = frame.meta.width as u32,
            h = frame.meta.height as u32
        )
        .entered();
        if frame.cpp != 1 {
            return Err(PipelineError::Unsupported(
                "gpu demosaic requires single-plane bayer frame".into(),
            ));
        }
        if let Some(pattern) = crate::cpu::demosaic::parse_xtrans(&frame.cfa_pattern) {
            return self.xtrans_to_texture(frame, &pattern);
        }
        if frame.cfa_pattern.len() != 4 {
            return Err(PipelineError::Unsupported(format!(
                "gpu demosaic requires a 2x2 or 6x6 CFA pattern, got '{}'",
                frame.cfa_pattern
            )));
        }
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.meta.width as u32;
        let h = frame.meta.height as u32;

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

        let bind = bind_group(
            device,
            "demosaic-bg",
            &self.passes.demosaic.layout,
            &[uniform_buf.as_entire_binding(), buf(&raw_buf), tex(&view)],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("demosaic-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "demosaic-pass",
            &self.passes.demosaic.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
            block: 1,
        }))
    }

    fn xtrans_to_texture(
        &self,
        frame: &RawFrame,
        pattern: &[u8; 36],
    ) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.xtrans",
            w = frame.meta.width as u32,
            h = frame.meta.height as u32
        )
        .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.meta.width as u32;
        let h = frame.meta.height as u32;

        let params = XtransParams {
            size: [w, h],
            _pad: [0, 0],
            pattern: xtrans_to_indices(pattern),
        };
        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), "xtrans-uniform");

        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("xtrans-raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });
        let green_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("xtrans-green-storage"),
            size: (frame.meta.width as u64) * (frame.meta.height as u64) * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
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

        let green_bind = bind_group(
            device,
            "xtrans-green-bg",
            &self.passes.xtrans.green.layout,
            &[
                uniform_buf.as_entire_binding(),
                buf(&raw_buf),
                buf(&green_buf),
            ],
        );
        let rgb_bind = bind_group(
            device,
            "xtrans-rgb-bg",
            &self.passes.xtrans.rgb.layout,
            &[
                uniform_buf.as_entire_binding(),
                buf(&raw_buf),
                buf(&green_buf),
                tex(&view),
            ],
        );

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("xtrans-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "xtrans-green-pass",
            &self.passes.xtrans.green.pipeline,
            &green_bind,
            gx,
            gy,
        );
        dispatch_2d(
            &mut encoder,
            "xtrans-rgb-pass",
            &self.passes.xtrans.rgb.pipeline,
            &rgb_bind,
            gx,
            gy,
        );
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
            block: 1,
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
            let bind = bind_group(
                device,
                "mipgen-bg",
                &self.passes.mipgen.layout,
                &[tex(&src_view), tex(&dst_view)],
            );
            dispatch_2d(
                encoder,
                "mipgen-pass",
                &self.passes.mipgen.pipeline,
                &bind,
                dst_w.div_ceil(16),
                dst_h.div_ceil(16),
            );
            mip_w = dst_w;
            mip_h = dst_h;
        }
    }
}
