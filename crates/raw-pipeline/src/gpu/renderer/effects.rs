use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, CommandEncoder, ComputePassDescriptor,
    TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::gpu::passes::effects_tone::EFFECTS_TONE_UNIFORM_SIZE;
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
            d.sharpen_amount as f32
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
        let out_view = out.texture.create_view(&TextureViewDescriptor::default());
        let mask_sharpen_view = out
            .mask_sharpen
            .create_view(&TextureViewDescriptor::default());

        let pass_h = &self.passes.output_sharpen;

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);

        let blur_uniform = |axis: u32| -> PooledUniform {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&sigma.to_ne_bytes());
            bytes[4..8].copy_from_slice(&radius.to_ne_bytes());
            bytes[8..12].copy_from_slice(&w.to_ne_bytes());
            bytes[12..16].copy_from_slice(&h.to_ne_bytes());
            bytes[16..20].copy_from_slice(&axis.to_ne_bytes());
            self.uniform_pool
                .acquire(device, queue, &bytes, "sharpen-blur-uniform")
        };
        let ub_h = blur_uniform(0);
        let bg_h = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-blur-h-bg"),
            layout: &pass_h.blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_h.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&linear_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_h_view),
                },
            ],
        });
        let ub_v = blur_uniform(1);
        let bg_v = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-blur-v-bg"),
            layout: &pass_h.blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_v.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&blur_h_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_full_view),
                },
            ],
        });
        {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sharpen-blur-h"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass_h.blur_pipeline);
            cp.set_bind_group(0, &bg_h, &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        }
        {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sharpen-blur-v"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass_h.blur_pipeline);
            cp.set_bind_group(0, &bg_v, &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        }

        let mut sh_bytes = [0u8; 48];
        sh_bytes[0..4].copy_from_slice(&amount.to_ne_bytes());
        sh_bytes[4..8].copy_from_slice(&detail_weight.to_ne_bytes());
        sh_bytes[8..12].copy_from_slice(&masking_thresh.to_ne_bytes());
        sh_bytes[12..16].copy_from_slice(&masking_softness.to_ne_bytes());
        sh_bytes[16..20].copy_from_slice(&w.to_ne_bytes());
        sh_bytes[20..24].copy_from_slice(&h.to_ne_bytes());
        sh_bytes[24..28].copy_from_slice(&use_mask.to_ne_bytes());
        sh_bytes[28..32].copy_from_slice(&preview_mode_u.to_ne_bytes());
        sh_bytes[32..36].copy_from_slice(&u32::from(masked_sharpen).to_ne_bytes());
        let ub_c = self
            .uniform_pool
            .acquire(device, queue, &sh_bytes, "sharpen-uniform");
        let bg_c = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-bg"),
            layout: &pass_h.sharpen_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_c.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&linear_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_full_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&out_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&sharpened_lin_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&mask_sharpen_view),
                },
            ],
        });
        let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("sharpen"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass_h.sharpen_pipeline);
        cp.set_bind_group(0, &bg_c, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
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
        dcp_active: bool,
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

        let mut bytes = [0u8; EFFECTS_TONE_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_ne_bytes());
        bytes[4..8].copy_from_slice(&h.to_ne_bytes());
        let vig_amount = (e.vignette_amount / 100.0) as f32;
        let vig_mid = (e.vignette_midpoint / 100.0) as f32;
        let vig_feather = (e.vignette_feather / 100.0) as f32;
        let vig_round = (e.vignette_roundness / 100.0) as f32;
        bytes[16..20].copy_from_slice(&vig_amount.to_ne_bytes());
        bytes[20..24].copy_from_slice(&vig_mid.to_ne_bytes());
        bytes[24..28].copy_from_slice(&vig_feather.to_ne_bytes());
        bytes[28..32].copy_from_slice(&vig_round.to_ne_bytes());
        let gr_amount = (e.grain_amount / 100.0) as f32;
        let gr_size = (e.grain_size / 100.0) as f32;
        let gr_rough = (e.grain_roughness / 100.0) as f32;
        bytes[32..36].copy_from_slice(&gr_amount.to_ne_bytes());
        bytes[36..40].copy_from_slice(&gr_size.to_ne_bytes());
        bytes[40..44].copy_from_slice(&gr_rough.to_ne_bytes());
        bytes[52..56].copy_from_slice(&(dcp_active as u32).to_ne_bytes());
        let p3 = matches!(color_space, crate::frame::OutputColorSpace::DisplayP3) as u32;
        bytes[56..60].copy_from_slice(&p3.to_ne_bytes());
        bytes[60..64].copy_from_slice(&warn_flags.to_ne_bytes());
        let r = roi.unwrap_or(crate::edits::CropRect::full());
        bytes[64..68].copy_from_slice(&r.x.to_ne_bytes());
        bytes[68..72].copy_from_slice(&r.y.to_ne_bytes());
        bytes[72..76].copy_from_slice(&r.w.to_ne_bytes());
        bytes[76..80].copy_from_slice(&r.h.to_ne_bytes());
        let ub = self
            .uniform_pool
            .acquire(device, queue, &bytes, "effects-tone-uniform");
        let src_binding = if sharpen_ran {
            BindingResource::TextureView(&sharpened_lin_view)
        } else {
            BindingResource::TextureView(&linear_view)
        };
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("effects-tone-bg"),
            layout: &pass.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: src_binding,
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&out_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&post_lin_view),
                },
            ],
        });
        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("effects-tone"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass.pipeline);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }
}
