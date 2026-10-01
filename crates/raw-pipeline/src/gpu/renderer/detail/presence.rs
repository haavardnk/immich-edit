use wgpu::{CommandEncoder, CommandEncoderDescriptor, Texture, TextureView};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{begin_pass, bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::presence::PresenceParams;
use crate::gpu::passes::sharpen::{SHARPEN_KERNEL_HALF, SharpenBlurParams};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::source::SourceExtent;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::gpu::uniform_pool::PooledUniform;
use crate::ops::blur::gaussian_kernel;
use crate::ops::presence::{PresenceBlur, presence_amounts, presence_blurs};

struct LevelBlur {
    _scratch: PooledTexture,
    _uniforms: [PooledUniform; 2],
    _binds: [wgpu::BindGroup; 2],
}

struct PyramidLabels {
    extract_bind: &'static str,
    mipgen_bind: &'static str,
    extract_pass: &'static str,
    mipgen_pass: &'static str,
}

const PRESENCE_PYRAMID: PyramidLabels = PyramidLabels {
    extract_bind: "luma-extract-bg",
    mipgen_bind: "pyramid-mipgen-bg",
    extract_pass: "luma-extract-pass",
    mipgen_pass: "pyramid-mipgen-pass",
};

const SHADOWS_PYRAMID: PyramidLabels = PyramidLabels {
    extract_bind: "luma-extract-bg-shadows",
    mipgen_bind: "pyramid-mipgen-bg-shadows",
    extract_pass: "luma-extract-shadows",
    mipgen_pass: "pyramid-mipgen-shadows",
};

impl GpuRenderer {
    fn encode_luma_pyramid(
        &self,
        encoder: &mut CommandEncoder,
        src_view: &TextureView,
        pyramid: &Texture,
        levels: u32,
        dims: (u32, u32),
        labels: &PyramidLabels,
    ) -> Vec<wgpu::BindGroup> {
        let device = &self.ctx.device;
        let (w, h) = dims;
        let level_views: Vec<TextureView> =
            (0..levels).map(|level| mip_view(pyramid, level)).collect();
        let extract_bind = bind_group(
            device,
            labels.extract_bind,
            &self.passes.luma_pyramid.extract_layout,
            &[tex(src_view), tex(&level_views[0])],
        );
        let mipgen_binds: Vec<wgpu::BindGroup> = (1..levels)
            .map(|level| {
                bind_group(
                    device,
                    labels.mipgen_bind,
                    &self.passes.mipgen.layout,
                    &[
                        tex(&level_views[(level - 1) as usize]),
                        tex(&level_views[level as usize]),
                    ],
                )
            })
            .collect();

        dispatch_2d(
            encoder,
            labels.extract_pass,
            &self.passes.luma_pyramid.extract_pipeline,
            &extract_bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        if !mipgen_binds.is_empty() {
            let mut pass = begin_pass(encoder, labels.mipgen_pass);
            pass.set_pipeline(&self.passes.mipgen.pipeline);
            let mut mw = w;
            let mut mh = h;
            for bg in &mipgen_binds {
                let dst_w = (mw / 2).max(1);
                let dst_h = (mh / 2).max(1);
                pass.set_bind_group(0, bg, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
                mw = dst_w;
                mh = dst_h;
            }
        }
        let mut retained = mipgen_binds;
        retained.push(extract_bind);
        retained
    }

    fn encode_level_blur(
        &self,
        encoder: &mut CommandEncoder,
        pyramid: &Texture,
        dims: (u32, u32),
        level: u32,
        blur: PresenceBlur,
    ) -> LevelBlur {
        let device = &self.ctx.device;
        let pass = &self.passes.output_sharpen;
        let size = [(dims.0 >> level).max(1), (dims.1 >> level).max(1)];
        let scratch = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, size[0], size[1], 1, STORAGE_SAMPLED),
            "presence-blur-scratch",
        );
        let level_view = mip_view(pyramid, level);
        let scratch_view = full_view(&scratch);
        let kernel = gaussian_kernel(blur.sigma);
        let radius = (kernel.len() / 2).min(SHARPEN_KERNEL_HALF - 1);
        let mut weights = [0.0f32; SHARPEN_KERNEL_HALF];
        for (slot, weight) in weights.iter_mut().zip(&kernel[kernel.len() / 2..]) {
            *slot = *weight;
        }
        let uniforms = [0, 1].map(|axis| {
            self.uniform(
                &SharpenBlurParams {
                    size,
                    radius: radius as u32,
                    axis,
                    weights,
                },
                "presence-blur-u",
            )
        });
        let binds = [
            bind_group(
                device,
                "presence-blur-h-bg",
                &pass.blur_layout,
                &[
                    uniforms[0].as_entire_binding(),
                    tex(&level_view),
                    tex(&scratch_view),
                ],
            ),
            bind_group(
                device,
                "presence-blur-v-bg",
                &pass.blur_layout,
                &[
                    uniforms[1].as_entire_binding(),
                    tex(&scratch_view),
                    tex(&level_view),
                ],
            ),
        ];
        {
            let mut cpass = begin_pass(encoder, "presence-blur-pass");
            cpass.set_pipeline(&pass.blur_pipeline);
            for bg in &binds {
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(size[0].div_ceil(16), size[1].div_ceil(16), 1);
            }
        }
        LevelBlur {
            _scratch: scratch,
            _uniforms: uniforms,
            _binds: binds,
        }
    }

    pub(in crate::gpu::renderer) fn submit_presence(
        &self,
        src: &Texture,
        extent: SourceExtent,
        edits: &Edits,
    ) -> PipelineResult<PooledTexture> {
        let dims = extent.dims;
        let _span = tracing::debug_span!("gpu.submit_presence", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let (fw, fh) = extent.full;
        let edits = edits.clamped();

        let blurs = presence_blurs(fw, fh);
        let pyramid_levels = blurs.levels().min(mip_count(w, h));
        let texture_level = blurs.texture.level.min(pyramid_levels - 1);
        let clarity_level = blurs.clarity.level.min(pyramid_levels - 1);

        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, w, h, pyramid_levels),
            "luma-pyramid",
        );
        let adjusted = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED),
            "presence-adjusted",
        );

        let amts = presence_amounts(&edits);
        let uniform_buf = self.uniform(
            &PresenceParams {
                size: [w, h],
                _pad0: [0; 2],
                amounts: [amts.texture, amts.clarity, amts.exposure, 0.0],
                mips: [texture_level, clarity_level, 0, 0],
            },
            "presence-uniform",
        );

        let src_view_full = full_view(src);
        let pyramid_full_view = full_view(&pyramid);
        let adjusted_view = full_view(&adjusted);
        let presence_bind = bind_group(
            device,
            "presence-bg",
            &self.passes.presence.adjust_layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view_full),
                tex(&pyramid_full_view),
                tex(&adjusted_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("presence-enc"),
        });
        let _pyramid_binds = self.encode_luma_pyramid(
            &mut encoder,
            &src_view_full,
            &pyramid,
            pyramid_levels,
            dims,
            &PRESENCE_PYRAMID,
        );
        let _blurs: Vec<LevelBlur> = [
            (amts.texture, texture_level, blurs.texture),
            (amts.clarity, clarity_level, blurs.clarity),
        ]
        .into_iter()
        .filter(|(amount, _, _)| *amount != 0.0)
        .map(|(_, level, blur)| self.encode_level_blur(&mut encoder, &pyramid, dims, level, blur))
        .collect();
        dispatch_2d(
            &mut encoder,
            "presence-adjust-pass",
            &self.passes.presence.adjust_pipeline,
            &presence_bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        queue.submit(Some(encoder.finish()));

        Ok(adjusted)
    }

    pub(in crate::gpu::renderer) fn submit_luma_pyramid(
        &self,
        src: &Texture,
        extent: SourceExtent,
    ) -> PipelineResult<PooledTexture> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let dims = extent.dims;
        let (w, h) = dims;
        let blurs = presence_blurs(extent.full.0, extent.full.1);
        let pyramid_levels = blurs.levels().min(mip_count(w, h));
        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, w, h, pyramid_levels),
            "shadows-pyramid",
        );
        let src_view = full_view(src);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("shadows-pyramid-enc"),
        });
        let _pyramid_binds = self.encode_luma_pyramid(
            &mut encoder,
            &src_view,
            &pyramid,
            pyramid_levels,
            dims,
            &SHADOWS_PYRAMID,
        );
        let _blur = self.encode_level_blur(
            &mut encoder,
            &pyramid,
            dims,
            blurs.shadows.level.min(pyramid_levels - 1),
            blurs.shadows,
        );
        queue.submit(Some(encoder.finish()));
        Ok(pyramid)
    }
}
