use std::hash::{Hash, Hasher};
use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoderDescriptor, TextureUsages};

use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::helpers::{
    DemosaicParams, SuperpixelParams, XtransParams, cfa_to_indices, mip_count, xtrans_to_indices,
};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d, write_texture_2d};
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
        let uniform_buf = self.uniform(&params, "superpixel-uniform");
        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("superpixel-raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });
        let texture = texture_2d(
            device,
            "linear-superpixel",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED,
        );
        let view = mip_view(&texture, 0);
        let bind = bind_group(
            device,
            "superpixel-bg",
            &self.passes.sensor_stage.demosaic.layout,
            &[uniform_buf.as_entire_binding(), buf(&raw_buf), tex(&view)],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("superpixel-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "superpixel-pass",
            &self.passes.sensor_stage.demosaic.superpixel,
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

        let texture = texture_2d(
            device,
            "linear-uploaded",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_DST,
        );
        write_texture_2d(
            queue,
            &texture,
            bytemuck::cast_slice(&rgba_f16),
            w * 8,
            (w, h),
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

        let uniform_buf = self.uniform(
            &DemosaicParams {
                size: [w, h],
                _pad: [0, 0],
                cfa: cfa_to_indices(&frame.cfa_pattern),
            },
            "demosaic-uniform",
        );

        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });

        let texture = texture_2d(
            device,
            "linear-cached",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED,
        );
        let view = mip_view(&texture, 0);

        let bind = bind_group(
            device,
            "demosaic-bg",
            &self.passes.sensor_stage.demosaic.layout,
            &[uniform_buf.as_entire_binding(), buf(&raw_buf), tex(&view)],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("demosaic-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "demosaic-pass",
            &self.passes.sensor_stage.demosaic.pipeline,
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

        let uniform_buf = self.uniform(
            &XtransParams {
                size: [w, h],
                _pad: [0, 0],
                pattern: xtrans_to_indices(pattern),
            },
            "xtrans-uniform",
        );

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

        let texture = texture_2d(
            device,
            "linear-cached",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED,
        );
        let view = mip_view(&texture, 0);

        let green_bind = bind_group(
            device,
            "xtrans-green-bg",
            &self.passes.sensor_stage.xtrans.green.layout,
            &[
                uniform_buf.as_entire_binding(),
                buf(&raw_buf),
                buf(&green_buf),
            ],
        );
        let rgb_bind = bind_group(
            device,
            "xtrans-rgb-bg",
            &self.passes.sensor_stage.xtrans.rgb.layout,
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
            &self.passes.sensor_stage.xtrans.green.pipeline,
            &green_bind,
            gx,
            gy,
        );
        dispatch_2d(
            &mut encoder,
            "xtrans-rgb-pass",
            &self.passes.sensor_stage.xtrans.rgb.pipeline,
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
}
