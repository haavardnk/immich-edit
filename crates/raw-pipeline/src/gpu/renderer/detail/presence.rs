use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Texture, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{begin_pass, bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::presence::PresenceParams;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::source::SourceExtent;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};

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
        let level_views: Vec<TextureView> = (0..levels)
            .map(|level| {
                pyramid.create_view(&TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
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

    pub(in crate::gpu::renderer) fn run_presence(
        &self,
        src: &Texture,
        extent: SourceExtent,
        edits: &Edits,
    ) -> PipelineResult<PooledTexture> {
        let dims = extent.dims;
        let _span = tracing::debug_span!("gpu.run_presence", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let (fw, fh) = extent.full;
        let edits = edits.clamped();

        let radii = presence_radii(fw, fh);
        let pyramid_levels = presence_pyramid_levels(fw, fh, radii).min(mip_count(w, h));

        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, w, h, pyramid_levels),
            "luma-pyramid",
        );
        let adjusted = self.texture_pool.acquire(
            device,
            TextureKey::new(
                self.ctx.linear_format,
                w,
                h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            ),
            "presence-adjusted",
        );

        let amts = presence_amounts(&edits);
        let mip_sel = presence_mips(fw, fh, radii);
        let params = PresenceParams {
            size: [w, h],
            _pad0: [0; 2],
            amounts: [amts.texture, amts.clarity, 0.0, 0.0],
            mips: [mip_sel.texture, mip_sel.clarity, 0, 0],
        };
        let uniform_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&params),
            "presence-uniform",
        );

        let src_view_full = src.create_view(&TextureViewDescriptor::default());
        let pyramid_full_view = pyramid.create_view(&TextureViewDescriptor::default());
        let adjusted_view = adjusted.create_view(&TextureViewDescriptor::default());
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

    pub(in crate::gpu::renderer) fn build_luma_pyramid(
        &self,
        src: &Texture,
        extent: SourceExtent,
    ) -> PipelineResult<PooledTexture> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let dims = extent.dims;
        let (w, h) = dims;
        let radii = presence_radii(extent.full.0, extent.full.1);
        let pyramid_levels =
            presence_pyramid_levels(extent.full.0, extent.full.1, radii).min(mip_count(w, h));
        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, w, h, pyramid_levels),
            "shadows-pyramid",
        );
        let src_view = src.create_view(&TextureViewDescriptor::default());
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
        queue.submit(Some(encoder.finish()));
        Ok(pyramid)
    }
}
