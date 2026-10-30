use wgpu::{BindGroup, CommandEncoder, Texture, TextureView};

use crate::gpu::dispatch::{begin_pass, bind_group, dispatch_2d, tex};
use crate::gpu::passes::luma_pyramid::LumaExtractParams;
use crate::gpu::passes::presence::{
    CLARITY_GUIDE_FORMAT, CLARITY_GUIDE_KERNEL_HALF, ClarityGuideParams,
};
use crate::gpu::passes::sharpen::{SHARPEN_KERNEL_HALF, SharpenBlurParams};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::texture::{STORAGE_SAMPLED, STORAGE_SAMPLED_SRC, full_view, mip_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::gpu::uniform_pool::PooledUniform;
use crate::ops::blur::gaussian_kernel;
use crate::ops::presence::{CLARITY_EDGE_EPS, EdgeAwareBlur, PresenceBlur};

pub(in crate::gpu::renderer) struct LevelBlur {
    _scratch: PooledTexture,
    _uniforms: [PooledUniform; 2],
    _binds: [BindGroup; 2],
}

pub(in crate::gpu::renderer) struct ClarityGuide {
    pub texture: PooledTexture,
    _scratch: PooledTexture,
    _uniforms: [PooledUniform; 6],
    _binds: [BindGroup; 6],
}

pub(in crate::gpu::renderer) struct LumaPyramid {
    _uniform: PooledUniform,
    _binds: Vec<BindGroup>,
}

pub(in crate::gpu::renderer) struct PyramidSource<'s> {
    pub view: &'s TextureView,
    pub offset: (u32, u32),
    pub dims: (u32, u32),
}

pub(in crate::gpu::renderer) struct PyramidLabels {
    pub extract_bind: &'static str,
    pub mipgen_bind: &'static str,
    pub extract_pass: &'static str,
    pub mipgen_pass: &'static str,
}

pub(in crate::gpu::renderer) const PRESENCE_PYRAMID: PyramidLabels = PyramidLabels {
    extract_bind: "luma-extract-bg",
    mipgen_bind: "pyramid-mipgen-bg",
    extract_pass: "luma-extract-pass",
    mipgen_pass: "pyramid-mipgen-pass",
};

pub(in crate::gpu::renderer) const SHADOWS_PYRAMID: PyramidLabels = PyramidLabels {
    extract_bind: "luma-extract-bg-shadows",
    mipgen_bind: "pyramid-mipgen-bg-shadows",
    extract_pass: "luma-extract-shadows",
    mipgen_pass: "pyramid-mipgen-shadows",
};

pub(in crate::gpu::renderer) fn level_dims((w, h): (u32, u32), level: u32) -> (u32, u32) {
    ((w >> level).max(1), (h >> level).max(1))
}

fn half_kernel<const N: usize>(sigma: f32) -> (u32, [f32; N]) {
    let kernel = gaussian_kernel(sigma);
    let radius = (kernel.len() / 2).min(N - 1);
    let mut weights = [0.0f32; N];
    for (slot, weight) in weights.iter_mut().zip(&kernel[kernel.len() / 2..]) {
        *slot = *weight;
    }
    (radius as u32, weights)
}

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn encode_luma_pyramid(
        &self,
        encoder: &mut CommandEncoder,
        src: PyramidSource<'_>,
        pyramid: &Texture,
        levels: u32,
        labels: &PyramidLabels,
    ) -> LumaPyramid {
        let device = &self.ctx.device;
        let (w, h) = src.dims;
        let uniform = self.uniform(
            &LumaExtractParams {
                offset: [src.offset.0, src.offset.1],
                _pad: [0; 2],
            },
            "luma-extract-u",
        );
        let level_views: Vec<TextureView> =
            (0..levels).map(|level| mip_view(pyramid, level)).collect();
        let extract_bind = bind_group(
            device,
            labels.extract_bind,
            &self.passes.luma_pyramid.extract_layout,
            &[
                uniform.as_entire_binding(),
                tex(src.view),
                tex(&level_views[0]),
            ],
        );
        let mipgen_binds: Vec<BindGroup> = (1..levels)
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
        let mut binds = mipgen_binds;
        binds.push(extract_bind);
        LumaPyramid {
            _uniform: uniform,
            _binds: binds,
        }
    }

    pub(in crate::gpu::renderer) fn encode_level_blur(
        &self,
        encoder: &mut CommandEncoder,
        level: &TextureView,
        (w, h): (u32, u32),
        blur: PresenceBlur,
    ) -> LevelBlur {
        let device = &self.ctx.device;
        let pass = &self.passes.output_sharpen;
        let size = [w, h];
        let scratch = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED),
            "presence-blur-scratch",
        );
        let scratch_view = full_view(&scratch);
        let (radius, weights) = half_kernel::<SHARPEN_KERNEL_HALF>(blur.sigma);
        let uniforms = [0, 1].map(|axis| {
            self.uniform(
                &SharpenBlurParams {
                    size,
                    radius,
                    axis,
                    weights,
                },
                "presence-blur-u",
            )
        });
        let io = [(level, &scratch_view), (&scratch_view, level)];
        let binds: [BindGroup; 2] = std::array::from_fn(|i| {
            bind_group(
                device,
                "presence-blur-bg",
                &pass.blur_layout,
                &[uniforms[i].as_entire_binding(), tex(io[i].0), tex(io[i].1)],
            )
        });
        {
            let mut cpass = begin_pass(encoder, "presence-blur-pass");
            cpass.set_pipeline(&pass.blur_pipeline);
            for bg in &binds {
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
            }
        }
        LevelBlur {
            _scratch: scratch,
            _uniforms: uniforms,
            _binds: binds,
        }
    }

    pub(in crate::gpu::renderer) fn encode_clarity_guide(
        &self,
        encoder: &mut CommandEncoder,
        level: &TextureView,
        (w, h): (u32, u32),
        blur: EdgeAwareBlur,
    ) -> ClarityGuide {
        let device = &self.ctx.device;
        let pass = &self.passes.presence;
        let size = [w, h];
        let key = TextureKey::new(CLARITY_GUIDE_FORMAT, w, h, 1, STORAGE_SAMPLED_SRC);
        let scratch = self
            .texture_pool
            .acquire(device, key, "clarity-guide-scratch");
        let texture = self.texture_pool.acquire(device, key, "clarity-guide");
        let scratch_view = full_view(&scratch);
        let guide_view = full_view(&texture);
        let prefilter = half_kernel::<CLARITY_GUIDE_KERNEL_HALF>(blur.prefilter);
        let window = half_kernel::<CLARITY_GUIDE_KERNEL_HALF>(blur.sigma);
        let uniforms = [0, 1, 2, 3, 4, 5].map(|mode| {
            let (radius, weights) = if mode < 2 { prefilter } else { window };
            self.uniform(
                &ClarityGuideParams {
                    size,
                    radius,
                    mode,
                    eps: CLARITY_EDGE_EPS,
                    _pad: [0; 3],
                    weights,
                },
                "clarity-guide-u",
            )
        });
        let io = [
            (level, &scratch_view),
            (&scratch_view, &guide_view),
            (&guide_view, &scratch_view),
            (&scratch_view, &guide_view),
            (&guide_view, &scratch_view),
            (&scratch_view, &guide_view),
        ];
        let binds: [BindGroup; 6] = std::array::from_fn(|i| {
            bind_group(
                device,
                "clarity-guide-bg",
                &pass.guide_layout,
                &[uniforms[i].as_entire_binding(), tex(io[i].0), tex(io[i].1)],
            )
        });
        {
            let mut cpass = begin_pass(encoder, "clarity-guide-pass");
            cpass.set_pipeline(&pass.guide_pipeline);
            for bg in &binds {
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
            }
        }
        ClarityGuide {
            texture,
            _scratch: scratch,
            _uniforms: uniforms,
            _binds: binds,
        }
    }
}
