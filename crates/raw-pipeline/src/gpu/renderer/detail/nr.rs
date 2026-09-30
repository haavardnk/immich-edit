mod chroma;
mod estimate;
mod luma;
mod profile;
mod sweep;

use std::sync::Arc;

use wgpu::{Texture, TextureUsages};

use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::gpu::helpers::mip_count;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::cache_keys::StageKeys;
use crate::gpu::renderer::stage_cache::Stage;
use crate::gpu::texture::{STORAGE_SAMPLED, texture_2d};
use crate::ops::denoise;
pub(in crate::gpu::renderer) use profile::NoiseProfile;
use sweep::NrSweep;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_nr(
        &self,
        src: &Arc<Texture>,
        dims: (u32, u32),
        edits: &Edits,
        keys: &StageKeys,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Arc<Texture>> {
        if let Some(t) = self.sensor.stages.get(Stage::Nr, keys.nr) {
            tracing::debug!(target: "gpu_cache", "nr_out cache hit");
            return Ok(t);
        }
        let (w, h) = dims;
        if w < 3 || h < 3 {
            return Ok(src.clone());
        }
        let profile = self.noise_profile_for(keys.nr_source, src, dims, cancel)?;
        let _span = tracing::debug_span!("gpu.submit_nr", w, h).entered();
        let d = &edits.detail;
        let chroma_levels = d.color_nr_active().then(|| {
            denoise::chroma::level_params(
                d.color_nr_amount as f32,
                d.color_nr_detail as f32,
                d.color_nr_smoothness as f32,
            )
        });
        let luma_levels = d.luma_nr_active().then(|| {
            denoise::luma::level_params(
                d.luma_nr_amount as f32,
                d.luma_nr_detail as f32,
                d.luma_nr_contrast as f32,
            )
        });
        let out = texture_2d(
            &self.ctx.device,
            "nr-out",
            self.ctx.linear_format,
            dims,
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_SRC,
        );
        let p = &self.passes.sensor_stage.nr;
        let mut sweep = NrSweep::new(self, "nr-enc");
        let between = (chroma_levels.is_some() && luma_levels.is_some())
            .then(|| sweep.scratch(self.ctx.linear_format, dims, "nr-chroma-out"));
        if let Some(levels) = chroma_levels {
            let dst: &Texture = between.as_deref().unwrap_or(&out);
            chroma::render(&mut sweep, p, [src, dst], dims, &profile, levels);
        }
        if let Some(levels) = luma_levels {
            let input: &Texture = between.as_deref().unwrap_or(src);
            luma::render(&mut sweep, p, [input, &out], dims, &profile.luma, levels);
        }
        self.encode_mipgen(sweep.encoder(), &out, w, h);
        sweep.submit();
        let out = Arc::new(out);
        self.sensor.stages.put(Stage::Nr, keys.nr, out.clone());
        Ok(out)
    }
}
