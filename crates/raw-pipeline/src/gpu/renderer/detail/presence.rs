use std::sync::Arc;

use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::presence::PresenceParams;
use crate::gpu::renderer::GpuRenderer;
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
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(labels.mipgen_pass),
                timestamp_writes: None,
            });
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
        dims: (u32, u32),
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let _span = tracing::debug_span!("gpu.run_presence", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let edits = edits.clamped();

        let radii = presence_radii(w, h);
        let pyramid_levels = presence_pyramid_levels(w, h, radii);

        let pyramid = LumaPyramidPass::allocate_pyramid(&self.ctx, w, h, pyramid_levels);
        let adjusted = device.create_texture(&TextureDescriptor {
            label: Some("presence-adjusted"),
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

        let amts = presence_amounts(&edits);
        let mip_sel = presence_mips(w, h, radii);
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
        let adjusted_mip0_view = adjusted.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let presence_bind = bind_group(
            device,
            "presence-bg",
            &self.passes.presence.adjust_layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view_full),
                tex(&pyramid_full_view),
                tex(&adjusted_mip0_view),
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
        self.encode_mipgen(&mut encoder, &adjusted, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(adjusted))
    }

    pub(in crate::gpu::renderer) fn build_luma_pyramid(
        &self,
        src: &Texture,
        dims: (u32, u32),
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let radii = presence_radii(w, h);
        let pyramid_levels = presence_pyramid_levels(w, h, radii);
        let pyramid = LumaPyramidPass::allocate_pyramid(&self.ctx, w, h, pyramid_levels);
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
        Ok(Arc::new(pyramid))
    }
}
