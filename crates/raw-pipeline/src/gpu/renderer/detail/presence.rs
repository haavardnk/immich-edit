use wgpu::{CommandEncoderDescriptor, Texture};

use super::pyramid::{PRESENCE_PYRAMID, PyramidSource, SHADOWS_PYRAMID, level_dims};
use crate::edits::Edits;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::presence::PresenceParams;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::display::{
    ContextNeeds, GridView, LevelView, Placed, PresenceLevels, ShadowLevels,
};
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::ops::presence::{presence_amounts, presence_blurs};
use crate::{PipelineError, PipelineResult};

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_presence_levels(
        &self,
        src: &Texture,
        placed: Placed,
        needs: ContextNeeds,
    ) -> (PresenceLevels, Vec<PooledTexture>) {
        let Placed { extent, origin } = placed;
        let dims = extent.dims;
        let _span =
            tracing::debug_span!("gpu.submit_presence_levels", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let blurs = presence_blurs(extent.full.0, extent.full.1);
        let pyramid_levels = blurs.levels().min(mip_count(dims.0, dims.1));
        let texture_level = blurs.texture.level.min(pyramid_levels - 1);
        let clarity_level = blurs.clarity.level.min(pyramid_levels - 1);
        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, dims.0, dims.1, pyramid_levels),
            "luma-pyramid",
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("presence-levels-enc"),
        });
        let source = PyramidSource {
            view: &full_view(src),
            offset: (0, 0),
            dims,
        };
        let _pyramid = self.encode_luma_pyramid(
            &mut encoder,
            source,
            &pyramid,
            pyramid_levels,
            &PRESENCE_PYRAMID,
        );
        let guide = needs.clarity.then(|| {
            let level = mip_view(&pyramid, clarity_level);
            let size = level_dims(dims, clarity_level);
            self.encode_clarity_guide(&mut encoder, &level, size, blurs.clarity)
        });
        let _texture_blur = needs.texture.then(|| {
            let level = mip_view(&pyramid, texture_level);
            let size = level_dims(dims, texture_level);
            self.encode_level_blur(&mut encoder, &level, size, blurs.texture)
        });
        self.ctx.queue.submit(Some(encoder.finish()));
        let local = |view, level| LevelView {
            grid: GridView::local(view, origin, level),
            level,
        };
        let levels = PresenceLevels {
            texture: needs
                .texture
                .then(|| local(mip_view(&pyramid, texture_level), texture_level)),
            guide: guide
                .as_ref()
                .map(|g| local(full_view(&g.texture), clarity_level)),
        };
        let held = std::iter::once(pyramid)
            .chain(guide.map(|g| g.texture))
            .collect();
        (levels, held)
    }

    pub(in crate::gpu::renderer) fn submit_presence_adjust(
        &self,
        base: &Texture,
        placed: Placed,
        levels: &PresenceLevels,
        edits: &Edits,
    ) -> PipelineResult<PooledTexture> {
        let Placed { extent, origin } = placed;
        let (w, h) = extent.dims;
        let _span = tracing::debug_span!("gpu.submit_presence_adjust", w, h).entered();
        let device = &self.ctx.device;
        let amts = presence_amounts(&edits.clamped());
        let texture = level_for(amts.texture, levels.texture.as_ref())?;
        let guide = level_for(amts.clarity, levels.guide.as_ref())?;
        let anchor = |level: Option<&LevelView>| level.map_or((0, 0), |l| l.grid.origin);
        let (tx, ty) = anchor(texture);
        let (gx, gy) = anchor(guide);
        let uniform = self.uniform(
            &PresenceParams {
                size: [w, h],
                origin: [origin.0, origin.1],
                amounts: [amts.texture, amts.clarity, amts.exposure, 0.0],
                mips: [
                    texture.map_or(0, |l| l.level),
                    guide.map_or(0, |l| l.level),
                    0,
                    0,
                ],
                anchors: [tx, ty, gx, gy],
            },
            "presence-uniform",
        );
        let adjusted = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED),
            "presence-adjusted",
        );
        let dummy = full_view(&self.dummy_texture);
        let view = |level: Option<&LevelView>| level.map_or(dummy.clone(), |l| l.grid.view.clone());
        let bind = bind_group(
            device,
            "presence-bg",
            &self.passes.presence.adjust_layout,
            &[
                uniform.as_entire_binding(),
                tex(&full_view(base)),
                tex(&view(texture)),
                tex(&full_view(&adjusted)),
                tex(&view(guide)),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("presence-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "presence-adjust-pass",
            &self.passes.presence.adjust_pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.ctx.queue.submit(Some(encoder.finish()));
        Ok(adjusted)
    }

    pub(in crate::gpu::renderer) fn submit_shadows_luma(
        &self,
        src: &Texture,
        placed: Placed,
        blur: Option<&LevelView>,
    ) -> (ShadowLevels, PooledTexture) {
        let Placed { extent, origin } = placed;
        let device = &self.ctx.device;
        let dims = extent.dims;
        let full = extent.full;
        let blurs = presence_blurs(full.0, full.1);
        let pyramid_levels = match blur {
            Some(_) => 1,
            None => blurs.levels().min(mip_count(dims.0, dims.1)),
        };
        let pyramid = self.texture_pool.acquire(
            device,
            LumaPyramidPass::pyramid_key(&self.ctx, dims.0, dims.1, pyramid_levels),
            "shadows-pyramid",
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("shadows-pyramid-enc"),
        });
        let source = PyramidSource {
            view: &full_view(src),
            offset: (0, 0),
            dims,
        };
        let _pyramid = self.encode_luma_pyramid(
            &mut encoder,
            source,
            &pyramid,
            pyramid_levels,
            &SHADOWS_PYRAMID,
        );
        let level = blurs.shadows.level.min(pyramid_levels - 1);
        let _local_blur = blur.is_none().then(|| {
            let view = mip_view(&pyramid, level);
            self.encode_level_blur(&mut encoder, &view, level_dims(dims, level), blurs.shadows)
        });
        self.ctx.queue.submit(Some(encoder.finish()));
        let luma = mip_view(&pyramid, 0);
        let shadows = match blur {
            Some(blur) => ShadowLevels {
                luma,
                blur: blur.grid.view.clone(),
                window: [0, 0, full.0, full.1],
                origin,
                level: blur.level,
            },
            None => ShadowLevels {
                luma,
                blur: mip_view(&pyramid, level),
                window: [origin.0, origin.1, full.0, full.1],
                origin: (0, 0),
                level,
            },
        };
        (shadows, pyramid)
    }
}

fn level_for(amount: f32, level: Option<&LevelView>) -> PipelineResult<Option<&LevelView>> {
    if amount == 0.0 {
        return Ok(None);
    }
    level
        .map(Some)
        .ok_or_else(|| PipelineError::Render("presence adjust has no level for its amount".into()))
}
