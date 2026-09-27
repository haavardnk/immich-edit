use parking_lot::MutexGuard;
use wgpu::CommandEncoder;

use super::DisplayTarget;
use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::{OutputColorSpace, PreviewMode, RenderOptions};
use crate::gpu::renderer::masks::Retained;
use crate::gpu::renderer::{GpuRenderer, pools};
use crate::gpu::resources::{OutputTargets, SharpenTargets};
use crate::gpu::texture_pool::PooledTexture;
use crate::ops::ResolvedDcp;

pub(super) struct FinishStage<'s> {
    pub edits: &'s Edits,
    pub opts: &'s RenderOptions,
    pub target: &'s OutputTargets,
    pub display: DisplayTarget<'s>,
    pub dcp: Option<&'s ResolvedDcp>,
    pub has_masks: bool,
}

impl GpuRenderer {
    pub(super) fn encode_finish<'a>(
        &'a self,
        encoder: &mut CommandEncoder,
        stage: FinishStage<'_>,
        scratch: &mut Vec<PooledTexture>,
        retained: &mut Retained,
    ) -> PipelineResult<Option<MutexGuard<'a, Vec<SharpenTargets>>>> {
        let FinishStage {
            edits,
            opts,
            target: p,
            display,
            dcp,
            has_masks,
        } = stage;
        let (out_w, out_h) = display.dims;
        let sharpen_preview = matches!(
            opts.preview_mode,
            PreviewMode::SharpenMask | PreviewMode::SharpenRadius | PreviewMode::SharpenDetail
        );
        let sharpen_active = edits.detail.sharpen_active();
        let p3_active = matches!(opts.output_color_space, OutputColorSpace::DisplayP3);
        let final_pass_active = sharpen_active
            || sharpen_preview
            || edits.effects.any_active()
            || has_masks
            || dcp.is_some()
            || p3_active
            || opts.gamut_warn
            || opts.clip_warn;
        let sharpen = final_pass_active
            .then(|| pools::acquire_target(&self.sharpen_pool, &self.ctx, out_w, out_h))
            .transpose()?;
        scratch.extend(self.encode_dcp_base_table(encoder, dcp, &p.linear_texture, display.dims));
        let Some(s) = sharpen.as_ref().map(|guard| &guard[0]) else {
            return Ok(sharpen);
        };
        let run_sharpen = sharpen_active || edits.masked_sharpen_active() || sharpen_preview;
        if run_sharpen {
            let uniforms =
                self.encode_sharpen(encoder, edits, p, s, display.dims, &opts.preview_mode);
            retained.uniforms.extend(uniforms);
        }
        let effects_src = if run_sharpen {
            &s.sharpened_lin
        } else {
            &p.linear_texture
        };
        let uniform = self.encode_effects_tone(encoder, edits, opts, effects_src, s, display);
        retained.uniforms.push(uniform);
        let warn_flags =
            opts.gamut_warn as u32 | ((opts.clip_warn as u32) << 1) | ((p3_active as u32) << 2);
        scratch.extend(self.encode_dcp_finish(encoder, dcp, &s.post_lin, display, warn_flags));
        Ok(sharpen)
    }
}
