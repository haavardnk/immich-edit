pub mod context;
pub mod pipeline;
pub mod readback;
pub mod uniforms;

use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, FilterMode, SamplerDescriptor,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::encode::encode_jpeg_rgba;
use crate::frame::{RawFrame, RenderOptions, RenderedImage, Renderer};
use crate::histogram::Histogram;
use crate::{PipelineError, PipelineResult};

use context::GpuContext;
use pipeline::GpuPipelines;
use readback::{copy_texture_to_buffer, make_readback_buffer, padded_row_bytes, read_rgba8};
use uniforms::{DemosaicParams, ProcessParams};

const CACHE_ITEMS: usize = 2;

struct CachedFrame {
    texture: Arc<Texture>,
    width: u32,
    height: u32,
}

pub struct GpuRenderer {
    ctx: Arc<GpuContext>,
    pipelines: Arc<GpuPipelines>,
    cache: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
}

impl GpuRenderer {
    pub fn new() -> PipelineResult<Self> {
        let ctx = GpuContext::new()?;
        let pipelines = Arc::new(GpuPipelines::new(&ctx));
        Ok(Self {
            ctx,
            pipelines,
            cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(CACHE_ITEMS).expect("nonzero"),
            )),
        })
    }

    pub fn adapter_label(&self) -> String {
        self.ctx.adapter_label()
    }

    fn frame_key(frame: &RawFrame) -> u64 {
        let ptr = frame.data.as_ptr() as usize as u64;
        let dims = ((frame.width as u64) << 32) | (frame.height as u64);
        ptr ^ dims
    }

    fn get_or_demosaic(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let key = Self::frame_key(frame);
        if let Some(c) = self.cache.lock().get(&key).cloned() {
            return Ok(c);
        }
        let cached = self.demosaic_to_texture(frame)?;
        self.cache.lock().put(key, cached.clone());
        Ok(cached)
    }

    fn demosaic_to_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
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
        let mut inv_range = [0.0f32; 4];
        for (i, slot) in inv_range.iter_mut().enumerate() {
            let r = frame.white_levels[i] - frame.black_levels[i];
            *slot = if r.abs() < 1e-6 { 0.0 } else { 1.0 / r };
        }
        let params = DemosaicParams {
            size: [w, h],
            _pad: [0, 0],
            cfa,
            black: frame.black_levels,
            inv_range,
        };

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("demosaic-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-cached"),
            size: Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("demosaic-bg"),
            layout: &self.pipelines.demosaic_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: raw_buf.as_entire_binding() },
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(&view) },
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
            pass.set_pipeline(&self.pipelines.demosaic_pipeline);
            pass.set_bind_group(0, &bind, &[]);
            let gx = w.div_ceil(16);
            let gy = h.div_ceil(16);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn process(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let edits = edits.clamped();

        let (oriented_w, oriented_h) = match edits.rotate {
            90 | 270 => (cached.height, cached.width),
            _ => (cached.width, cached.height),
        };
        let crop = edits
            .crop
            .as_ref()
            .map(|c| {
                let cw = (c.width as f32).max(0.0001);
                let ch = (c.height as f32).max(0.0001);
                [c.x as f32, c.y as f32, cw, ch]
            })
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);

        let cropped_w = ((oriented_w as f32) * crop[2]).max(1.0) as u32;
        let cropped_h = ((oriented_h as f32) * crop[3]).max(1.0) as u32;

        let (out_w, out_h) = scale_to_max(cropped_w, cropped_h, opts.max_edge);

        let mut wb = compute_wb(frame.wb_coeffs, edits.wb_temp as f32, edits.wb_tint as f32);
        wb[3] = 1.0;

        let exposure = 2.0f32.powf(edits.exposure_ev as f32);
        let contrast = (edits.contrast as f32) / 100.0;
        let hl = (edits.highlights as f32) / 100.0;
        let sh = (edits.shadows as f32) / 100.0;
        let sat = (edits.saturation as f32) / 100.0;

        let params = ProcessParams {
            src_size: [cached.width, cached.height],
            out_size: [out_w, out_h],
            crop,
            wb,
            tone: [exposure, contrast, hl, sh],
            flags: [edits.rotate as u32, edits.flip_h as u32, edits.flip_v as u32, 0],
            sat,
            _pad: [0.0, 0.0, 0.0],
        };

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("process-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let src_view = cached.texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("linear-samp"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let out_texture = device.create_texture(&TextureDescriptor {
            label: Some("output"),
            size: Extent3d { width: out_w, height: out_h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let out_view = out_texture.create_view(&TextureViewDescriptor::default());

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("process-bg"),
            layout: &self.pipelines.process_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&src_view) },
                BindGroupEntry { binding: 2, resource: BindingResource::Sampler(&sampler) },
                BindGroupEntry { binding: 3, resource: BindingResource::TextureView(&out_view) },
            ],
        });

        let readback = make_readback_buffer(device, out_w, out_h);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("process-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("process-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipelines.process_pipeline);
            pass.set_bind_group(0, &bind, &[]);
            let gx = out_w.div_ceil(16);
            let gy = out_h.div_ceil(16);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        copy_texture_to_buffer(&mut encoder, &out_texture, &readback, out_w, out_h);
        queue.submit(Some(encoder.finish()));

        let rgba = read_rgba8(&self.ctx, &readback, out_w, out_h)?;
        let _ = padded_row_bytes(out_w);

        let histogram = Histogram::from_rgba8(&rgba);
        let jpeg = encode_jpeg_rgba(&rgba, out_w, out_h, 85)?;

        Ok(RenderedImage {
            jpeg,
            histogram,
            width: out_w,
            height: out_h,
            renderer: "gpu".into(),
        })
    }
}

impl Renderer for GpuRenderer {
    fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        let cached = self.get_or_demosaic(frame)?;
        self.process(&cached, frame, edits, options)
    }

    fn name(&self) -> &str {
        "gpu"
    }
}

fn cfa_to_indices(pattern: &str) -> [u32; 4] {
    let mut out = [1u32; 4];
    for (i, c) in pattern.chars().take(4).enumerate() {
        out[i] = match c {
            'R' => 0,
            'G' => 1,
            'B' => 2,
            _ => 1,
        };
    }
    out
}

fn compute_wb(raw: [f32; 4], temp: f32, tint: f32) -> [f32; 4] {
    let mut c = raw;
    if c[0] == 0.0 && c[1] == 0.0 && c[2] == 0.0 {
        c = [1.0, 1.0, 1.0, 1.0];
    }
    if c[1] > 0.0 {
        c[0] /= c[1];
        c[2] /= c[1];
        c[3] /= c[1];
        c[1] = 1.0;
    }
    let t = temp / 100.0;
    let ti = tint / 100.0;
    c[0] *= 1.0 + t * 0.5;
    c[2] *= 1.0 - t * 0.5;
    c[1] *= 1.0 - ti * 0.3;
    c
}

fn scale_to_max(w: u32, h: u32, max_edge: u32) -> (u32, u32) {
    if w <= max_edge && h <= max_edge {
        return (w, h);
    }
    let scale = max_edge as f64 / w.max(h) as f64;
    let nw = ((w as f64) * scale).round() as u32;
    let nh = ((h as f64) * scale).round() as u32;
    (nw.max(1), nh.max(1))
}
