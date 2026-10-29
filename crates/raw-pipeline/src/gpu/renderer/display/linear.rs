use std::collections::HashMap;
use std::sync::Arc;

use wgpu::Texture;

use super::StageState;
use crate::cancel::CancelToken;
use crate::cpu::masked::effective_edits_for_layer;
use crate::edits::{Edits, MaskLayer};
use crate::frame::{FrameMeta, RenderOptions};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::detail::LensBlurInput;
use crate::gpu::source::SourceExtent;
use crate::gpu::timer::RenderTimings;
use crate::ops::lens_blur::LensBlur;
use crate::ops::presence::{PresenceAmounts, presence_amounts};
use crate::timing;
use crate::{PipelineError, PipelineResult};

impl GpuRenderer {
    pub(super) fn dehaze_stage(
        &self,
        state: &mut StageState,
        edits: &Edits,
        t: &RenderTimings,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<()> {
        if edits.basic.dehaze == 0.0 {
            return Ok(());
        }
        let atmosphere = state.atmosphere.ok_or_else(|| {
            PipelineError::Unsupported("dehaze needs an atmosphere estimate on the source".into())
        })?;
        let dims = state.extent.dims;
        let tex = t.stage(timing::DEHAZE, || {
            let _span = tracing::debug_span!("gpu_dehaze", w = dims.0, h = dims.1).entered();
            self.submit_dehaze(&state.texture, state.extent, edits, atmosphere)
        })?;
        crate::cancel::check(cancel)?;
        state.texture = state.hold(tex);
        Ok(())
    }

    pub(super) fn lens_blur_stage(
        &self,
        state: &mut StageState,
        meta: &FrameMeta,
        edits: &Edits,
        opts: &RenderOptions,
        t: &RenderTimings,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<()> {
        let Some(blur) = LensBlur::resolve(edits, &opts.rasters, state.extent.full) else {
            return Ok(());
        };
        let input = LensBlurInput {
            extent: state.extent,
            origin: state.origin,
            orientation: meta.orientation,
            blur: &blur,
            lens: &edits.lens,
        };
        let tex = t.stage(timing::LENS_BLUR, || {
            self.submit_lens_blur(&state.texture, input)
        })?;
        crate::cancel::check(cancel)?;
        state.texture = state.hold(tex);
        Ok(())
    }

    pub(super) fn presence_stage(
        &self,
        state: &mut StageState,
        edits: &Edits,
        t: &RenderTimings,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<()> {
        if edits.basic.texture == 0.0 && edits.basic.clarity == 0.0 {
            return Ok(());
        }
        let tex = t.stage(timing::PRESENCE, || {
            self.submit_presence(&state.texture, state.extent, edits)
        })?;
        crate::cancel::check(cancel)?;
        state.texture = state.hold(tex);
        Ok(())
    }

    pub(super) fn layer_presence_stage(
        &self,
        state: &mut StageState,
        base: &Arc<Texture>,
        edits: &Edits,
        t: &RenderTimings,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<()> {
        let extent = state.extent;
        let mut cache: HashMap<[u32; 3], Arc<Texture>> = HashMap::new();
        for (layer, eff, amts) in layer_presence(edits) {
            let key = amts.bits();
            let tex = match cache.get(&key) {
                Some(tex) => tex.clone(),
                None if !amts.adjusts() => base.clone(),
                None => {
                    let tex = t.stage(timing::PRESENCE, || {
                        self.submit_presence(base, extent, &eff)
                    })?;
                    crate::cancel::check(cancel)?;
                    state.hold(tex)
                }
            };
            cache.insert(key, tex.clone());
            state.layers.insert(layer.id.clone(), tex);
        }
        Ok(())
    }

    pub(super) fn shadows_stage(
        &self,
        state: &mut StageState,
        edits: &Edits,
        t: &RenderTimings,
    ) -> PipelineResult<()> {
        if !crate::ops::presence::has_shadows(edits) {
            return Ok(());
        }
        let pyramid = t.stage(timing::SHADOWS, || {
            self.submit_luma_pyramid(&state.texture, state.extent)
        })?;
        state.shadows = Some(state.hold(pyramid));
        Ok(())
    }

    pub(super) fn resample_stage(
        &self,
        state: &mut StageState,
        meta: &FrameMeta,
        edits: &Edits,
        out_dims: (u32, u32),
        t: &RenderTimings,
    ) -> PipelineResult<()> {
        let SourceExtent { dims, full } = state.extent;
        let (crop_w, crop_h) = crate::geom::display_crop_px(meta.orientation, edits, full);
        let ratio = (crop_w as f32 / out_dims.0 as f32).max(crop_h as f32 / out_dims.1 as f32);
        t.stage(timing::RESAMPLE, || {
            let Some(target) = crate::geom::resample_target(full, ratio) else {
                return Ok(());
            };
            let work_dims = scaled_dims(dims, full, target);
            let texture =
                self.resample_lanczos(&state.texture, dims, work_dims, "process-downscale", false)?;
            let layers = state
                .layers
                .iter()
                .map(|(id, tex)| {
                    self.resample_lanczos(tex, dims, work_dims, "layer-downscale", false)
                        .map(|t| (id.clone(), t))
                })
                .collect::<PipelineResult<HashMap<String, Arc<Texture>>>>()?;
            state.texture = texture;
            state.layers = layers;
            state.extent = SourceExtent {
                dims: work_dims,
                full: target,
            };
            PipelineResult::Ok(())
        })
    }
}

pub(in crate::gpu::renderer) fn layer_presence(
    edits: &Edits,
) -> Vec<(&MaskLayer, Edits, PresenceAmounts)> {
    let global = presence_amounts(edits);
    edits
        .masks
        .iter()
        .filter(|layer| layer.is_effective())
        .filter_map(|layer| {
            let eff = effective_edits_for_layer(edits, layer);
            let amts = presence_amounts(&eff);
            (amts != global).then_some((layer, eff, amts))
        })
        .collect()
}

pub(in crate::gpu::renderer) fn scaled_dims(
    dims: (u32, u32),
    full: (u32, u32),
    target: (u32, u32),
) -> (u32, u32) {
    if dims == full {
        return target;
    }
    let scale = |d: u32, f: u32, t: u32| {
        ((u64::from(d) * u64::from(t) + u64::from(f) / 2) / u64::from(f)).max(1) as u32
    };
    (
        scale(dims.0, full.0, target.0),
        scale(dims.1, full.1, target.1),
    )
}
