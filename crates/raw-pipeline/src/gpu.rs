pub mod context;
pub mod pipeline;
pub mod readback;
pub mod shader_builder;

use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, FilterMode, SamplerDescriptor,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::encode::encode_jpeg_rgba;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use crate::ops::OpContext;
use crate::{PipelineError, PipelineResult};

use context::GpuContext;
use pipeline::GpuPipelines;
use readback::{
    copy_texture_to_buffer, make_readback_buffer, make_readback_buffer_f16, read_rgba8,
    read_rgba16f_as_rgb,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DemosaicParams {
    size: [u32; 2],
    _pad: [u32; 2],
    cfa: [u32; 4],
}

const CACHE_ITEMS: usize = 2;

struct CachedFrame {
    texture: Arc<Texture>,
    width: u32,
    height: u32,
}

struct OutputPool {
    texture: Texture,
    readback: Buffer,
    linear_texture: Texture,
    linear_readback: Buffer,
    alloc_w: u32,
    alloc_h: u32,
}

fn round_up_256(v: u32) -> u32 {
    (v + 255) & !255
}

fn mip_count(w: u32, h: u32) -> u32 {
    (w.max(h) as f32).log2().floor() as u32 + 1
}

pub struct GpuRenderer {
    ctx: Arc<GpuContext>,
    pipelines: Arc<GpuPipelines>,
    cache: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    output_pool: Mutex<Option<OutputPool>>,
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
            output_pool: Mutex::new(None),
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
        let cached = if frame.cpp == 3 {
            self.upload_rgb_texture(frame)?
        } else {
            self.demosaic_to_texture(frame)?
        };
        self.cache.lock().put(key, cached.clone());
        Ok(cached)
    }

    fn upload_rgb_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
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
            wgpu::ImageDataLayout {
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

        self.encode_mipgen(&texture, w, h);

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
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
        let params = DemosaicParams {
            size: [w, h],
            _pad: [0, 0],
            cfa,
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
            layout: &self.pipelines.demosaic_layout,
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
            pass.set_pipeline(&self.pipelines.demosaic_pipeline);
            pass.set_bind_group(0, &bind, &[]);
            let gx = w.div_ceil(16);
            let gy = h.div_ceil(16);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit(Some(encoder.finish()));
        self.encode_mipgen(&texture, w, h);

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn encode_mipgen(&self, texture: &Texture, w: u32, h: u32) {
        let levels = mip_count(w, h);
        if levels <= 1 {
            return;
        }
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("mipgen-enc"),
        });
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
                layout: &self.pipelines.mipgen_layout,
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
                pass.set_pipeline(&self.pipelines.mipgen_pipeline);
                pass.set_bind_group(0, &bind, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
            }
            mip_w = dst_w;
            mip_h = dst_h;
        }
        queue.submit(Some(encoder.finish()));
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

        for op in self.pipelines.registry.active(&edits) {
            if op.gpu().is_none() {
                return Err(PipelineError::Unsupported(format!(
                    "gpu pipeline missing op: {}",
                    op.id()
                )));
            }
        }

        let (sensor_w, sensor_h) = (cached.width, cached.height);
        let (display_w, display_h) = if frame.orientation.0 {
            (sensor_h, sensor_w)
        } else {
            (sensor_w, sensor_h)
        };

        let (oriented_w, oriented_h) = match edits.geometry.rotate {
            90 | 270 => (display_h, display_w),
            _ => (display_w, display_h),
        };

        let (out_w, out_h) = scale_to_max(oriented_w, oriented_h, opts.max_edge);

        let src_max = cached.width.max(cached.height) as f32;
        let out_max = out_w.max(out_h) as f32;
        let lod = if src_max > out_max {
            (src_max / out_max).log2()
        } else {
            0.0
        };

        let (ot, oh_h, oh_v) = frame.orientation;
        let orient_packed = (oh_h as u32) | ((oh_v as u32) << 1) | ((ot as u32) << 2);

        let xyz_to_cam = if frame.color_matrices.len() >= 2 {
            let cct = crate::color::estimate_scene_cct(
                frame.wb_coeffs,
                &frame.color_matrices.last().unwrap().1,
            );
            crate::color::interpolate_xyz_to_cam(&frame.color_matrices, cct)
        } else {
            frame.xyz_to_cam
        };
        let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&xyz_to_cam) {
            crate::color::cam_to_srgb_matrix(xyz_to_cam)
        } else {
            crate::color::identity_3x3()
        };
        let ctx_op = OpContext {
            wb_coeffs: frame.wb_coeffs,
            cam_to_srgb,
            is_raw: frame.is_raw,
        };
        let built = &self.pipelines.built;
        let registry = &self.pipelines.registry;
        let mut uniform_bytes = vec![0u8; built.uniform_size];
        write_header(
            &mut uniform_bytes,
            [cached.width, cached.height],
            [out_w, out_h],
            [0.0, 0.0, 1.0, 1.0],
            [
                edits.geometry.rotate as u32,
                edits.geometry.flip_h as u32,
                edits.geometry.flip_v as u32,
                orient_packed,
            ],
            [lod, 0.0, 0.0, 0.0],
        );
        let mut active_mask: u32 = 0;
        for slot in &built.color_ops {
            let op = &registry.ops()[slot.op_index];
            if op.is_active(&edits) {
                active_mask |= 1u32 << slot.active_bit;
            }
            let mut buf = vec![0.0f32; slot.vec4_count * 4];
            op.write_gpu_uniform(&edits, &ctx_op, &mut buf);
            let off = slot.uniform_offset;
            let bytes = slot.vec4_count * 16;
            uniform_bytes[off..off + bytes].copy_from_slice(bytemuck::cast_slice(&buf));
        }
        let mask_words: [u32; 4] = [active_mask, 0, 0, 0];
        uniform_bytes[crate::gpu::shader_builder::ACTIVE_MASK_OFFSET
            ..crate::gpu::shader_builder::ACTIVE_MASK_OFFSET + 16]
            .copy_from_slice(bytemuck::cast_slice(&mask_words));

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("process-uniform"),
            contents: &uniform_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let src_view = cached
            .texture
            .create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("linear-samp"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let mut pool = self.output_pool.lock();
        let need_w = round_up_256(out_w);
        let need_h = round_up_256(out_h);
        let fits = pool
            .as_ref()
            .is_some_and(|p| p.alloc_w >= need_w && p.alloc_h >= need_h);
        if !fits {
            *pool = Some(OutputPool {
                texture: device.create_texture(&TextureDescriptor {
                    label: Some("output"),
                    size: Extent3d {
                        width: need_w,
                        height: need_h,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
                    view_formats: &[],
                }),
                readback: make_readback_buffer(device, need_w, need_h),
                linear_texture: device.create_texture(&TextureDescriptor {
                    label: Some("linear-output"),
                    size: Extent3d {
                        width: need_w,
                        height: need_h,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
                    view_formats: &[],
                }),
                linear_readback: make_readback_buffer_f16(device, need_w, need_h),
                alloc_w: need_w,
                alloc_h: need_h,
            });
        }
        let p = pool.as_ref().expect("pool populated");
        let out_view = p.texture.create_view(&TextureViewDescriptor::default());
        let linear_view = p
            .linear_texture
            .create_view(&TextureViewDescriptor::default());

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("process-bg"),
            layout: &self.pipelines.process_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&out_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&linear_view),
                },
            ],
        });

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
        copy_texture_to_buffer(&mut encoder, &p.texture, &p.readback, out_w, out_h);
        copy_texture_to_buffer(
            &mut encoder,
            &p.linear_texture,
            &p.linear_readback,
            out_w,
            out_h,
        );
        queue.submit(Some(encoder.finish()));

        let rgba = read_rgba8(&self.ctx, &p.readback, out_w, out_h)?;
        let linear_rgb = read_rgba16f_as_rgb(&self.ctx, &p.linear_readback, out_w, out_h)?;
        drop(pool);

        let ((histogram, linear_histogram), jpeg) = rayon::join(
            || {
                rayon::join(
                    || Histogram::from_rgba8(&rgba),
                    || Histogram::from_rgb(&linear_rgb, out_w as usize, out_h as usize),
                )
            },
            || encode_jpeg_rgba(&rgba, out_w, out_h, 85),
        );
        let jpeg = jpeg?;

        Ok(RenderedImage {
            jpeg,
            histogram,
            linear_histogram: Some(linear_histogram),
            width: out_w,
            height: out_h,
            renderer: "gpu".into(),
        })
    }

    pub fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        self.render_with_cancel(frame, edits, options, None)
    }

    pub fn render_with_cancel(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        crate::cancel::check(cancel)?;
        let cached = self.get_or_demosaic(frame)?;
        crate::cancel::check(cancel)?;
        let out = self.process(&cached, frame, edits, options)?;
        crate::cancel::check(cancel)?;
        Ok(out)
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

fn write_header(
    dst: &mut [u8],
    src_size: [u32; 2],
    out_size: [u32; 2],
    crop: [f32; 4],
    flags: [u32; 4],
    geom_extra: [f32; 4],
) {
    dst[0..8].copy_from_slice(bytemuck::cast_slice(&src_size));
    dst[8..16].copy_from_slice(bytemuck::cast_slice(&out_size));
    dst[16..32].copy_from_slice(bytemuck::cast_slice(&crop));
    dst[32..48].copy_from_slice(bytemuck::cast_slice(&flags));
    dst[48..64].copy_from_slice(bytemuck::cast_slice(&geom_extra));
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
