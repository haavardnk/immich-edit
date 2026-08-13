use wgpu::{BindGroupEntry, CommandEncoder, TextureViewDescriptor};

use crate::edits::Edits;
use crate::gpu::dispatch::{bind_group, bind_group_indexed, dispatch_2d, tex};
use crate::gpu::passes::effects_tone::EffectsToneParams;
use crate::gpu::passes::sharpen::{SharpenBlurParams, SharpenParams};
use crate::gpu::resources::{OutputTargets, SharpenTargets};
use crate::gpu::uniform_pool::PooledUniform;

use super::GpuRenderer;

impl GpuRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_sharpen(
        &self,
        encoder: &mut CommandEncoder,
        edits: &Edits,
        out: &OutputTargets,
        sh: &SharpenTargets,
        w: u32,
        h: u32,
        preview: &crate::frame::PreviewMode,
        masked_sharpen: bool,
    ) {
        let _span = tracing::debug_span!("gpu.encode_sharpen", w = w, h = h).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let d = &edits.detail;
        let sigma = (d.sharpen_radius as f32).max(0.01);
        let radius = (sigma * 3.0).ceil();
        let sharpen_active = d.sharpen_active();
        let amount = if sharpen_active {
            d.sharpen_amount.unwrap_or(0.0) as f32
        } else {
            0.0
        };
        let detail_weight = 0.5 + 0.5 * (d.sharpen_detail as f32 / 100.0);
        let masking = (d.sharpen_masking as f32 / 100.0).clamp(0.0, 1.0);
        let preview_mode_u: u32 = match preview {
            crate::frame::PreviewMode::None => 0,
            crate::frame::PreviewMode::SharpenMask => 1,
            crate::frame::PreviewMode::SharpenRadius => 2,
            crate::frame::PreviewMode::SharpenDetail => 3,
            crate::frame::PreviewMode::MaskWeight { .. } => 0,
        };
        let use_mask =
            if ((sharpen_active || masked_sharpen) && masking > 0.0) || preview_mode_u == 1 {
                1u32
            } else {
                0u32
            };
        let masking_thresh = masking * 0.15;
        let masking_softness = 0.15f32;

        let linear_view = out
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let blur_h_view = sh.blur_h.create_view(&TextureViewDescriptor::default());
        let blur_full_view = sh.blur_full.create_view(&TextureViewDescriptor::default());
        let sharpened_lin_view = sh
            .sharpened_lin
            .create_view(&TextureViewDescriptor::default());
        let mask_sharpen_view = out
            .mask_sharpen
            .create_view(&TextureViewDescriptor::default());

        let pass_h = &self.passes.output_sharpen;

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);

        let blur_uniform = |axis: u32| -> PooledUniform {
            let params = SharpenBlurParams {
                sigma,
                radius,
                size: [w, h],
                axis,
                _pad: [0; 3],
            };
            self.uniform_pool.acquire(
                device,
                queue,
                bytemuck::bytes_of(&params),
                "sharpen-blur-uniform",
            )
        };
        let ub_h = blur_uniform(0);
        let bg_h = bind_group(
            device,
            "sharpen-blur-h-bg",
            &pass_h.blur_layout,
            &[
                ub_h.as_entire_binding(),
                tex(&linear_view),
                tex(&blur_h_view),
            ],
        );
        let ub_v = blur_uniform(1);
        let bg_v = bind_group(
            device,
            "sharpen-blur-v-bg",
            &pass_h.blur_layout,
            &[
                ub_v.as_entire_binding(),
                tex(&blur_h_view),
                tex(&blur_full_view),
            ],
        );
        dispatch_2d(
            encoder,
            "sharpen-blur-h",
            &pass_h.blur_pipeline,
            &bg_h,
            gx,
            gy,
        );
        dispatch_2d(
            encoder,
            "sharpen-blur-v",
            &pass_h.blur_pipeline,
            &bg_v,
            gx,
            gy,
        );

        let sharpen_params = SharpenParams {
            amount,
            detail_weight,
            masking_thresh,
            masking_softness,
            size: [w, h],
            use_mask,
            preview_mode: preview_mode_u,
            masked_sharpen: u32::from(masked_sharpen),
            _pad: [0; 3],
        };
        let ub_c = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&sharpen_params),
            "sharpen-uniform",
        );
        let bg_c = bind_group_indexed(
            device,
            "sharpen-bg",
            &pass_h.sharpen_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_c.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: tex(&linear_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: tex(&blur_full_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: tex(&sharpened_lin_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: tex(&mask_sharpen_view),
                },
            ],
        );
        dispatch_2d(encoder, "sharpen", &pass_h.sharpen_pipeline, &bg_c, gx, gy);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_effects_tone(
        &self,
        encoder: &mut CommandEncoder,
        edits: &Edits,
        out: &OutputTargets,
        sh: &SharpenTargets,
        w: u32,
        h: u32,
        sharpen_ran: bool,
        color_space: crate::frame::OutputColorSpace,
        warn_flags: u32,
        roi: Option<crate::edits::CropRect>,
    ) {
        let _span = tracing::debug_span!("gpu.encode_effects_tone", w = w, h = h).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let e = &edits.effects;
        let pass = &self.passes.effects_tone;

        let linear_view = out
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let sharpened_lin_view = sh
            .sharpened_lin
            .create_view(&TextureViewDescriptor::default());
        let post_lin_view = sh.post_lin.create_view(&TextureViewDescriptor::default());
        let out_view = out.texture.create_view(&TextureViewDescriptor::default());

        let r = roi.unwrap_or(crate::edits::CropRect::full());
        let params = EffectsToneParams {
            size: [w, h],
            _pad0: [0; 2],
            vignette: [
                (e.vignette_amount / 100.0) as f32,
                (e.vignette_midpoint / 100.0) as f32,
                (e.vignette_feather / 100.0) as f32,
                (e.vignette_roundness / 100.0) as f32,
            ],
            grain: [
                (e.grain_amount / 100.0) as f32,
                (e.grain_size / 100.0) as f32,
                (e.grain_roughness / 100.0) as f32,
            ],
            _pad1: [0.0; 3],
            display_p3: matches!(color_space, crate::frame::OutputColorSpace::DisplayP3) as u32,
            warn_flags,
            roi: [r.x, r.y, r.w, r.h],
        };
        let ub = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&params),
            "effects-tone-uniform",
        );
        let src_binding = if sharpen_ran {
            tex(&sharpened_lin_view)
        } else {
            tex(&linear_view)
        };
        let bg = bind_group(
            device,
            "effects-tone-bg",
            &pass.layout,
            &[
                ub.as_entire_binding(),
                src_binding,
                tex(&out_view),
                tex(&post_lin_view),
            ],
        );
        dispatch_2d(
            encoder,
            "effects-tone",
            &pass.pipeline,
            &bg,
            w.div_ceil(16),
            h.div_ceil(16),
        );
    }
}
