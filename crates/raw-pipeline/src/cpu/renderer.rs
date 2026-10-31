use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::PipelineResult;
use crate::admission::{Admitted, HostMemory};
use crate::ai_denoise::DenoisePlane;
use crate::cancel::CancelToken;
use crate::dcp::setup::DcpSetup;
use crate::edits::Edits;
use crate::frame::{PreviewMode, RawFrame, RenderOptions, RenderedImage};

mod sensor_cache;
#[cfg(test)]
mod tests;

use sensor_cache::SensorCache;
pub(crate) use sensor_cache::SensorStage;

pub struct CpuRenderer {
    sensor_cache: Arc<SensorCache>,
    host: HostMemory,
}

impl Default for CpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self::with_host(HostMemory::default())
    }

    pub fn with_host(host: HostMemory) -> Self {
        let sensor_cache = Arc::new(SensorCache::default());
        host.register(sensor_cache.clone());
        Self { sensor_cache, host }
    }

    pub fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> crate::PipelineResult<RenderedImage> {
        self.render_with_cancel(frame, edits, options, None)
    }

    pub fn render_with_cancel(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&CancelToken>,
    ) -> crate::PipelineResult<RenderedImage> {
        crate::cpu::pipeline::render_cached(frame, edits, options, cancel, Some(self))
    }

    pub fn render_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&CancelToken>,
    ) -> crate::PipelineResult<crate::source::RenderedSource> {
        crate::cpu::pipeline::render_source(frame, edits, options, cancel, self)
    }

    pub(crate) fn admit(
        &self,
        bytes: u64,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        self.host.admit(bytes, cancel)
    }

    pub(crate) fn get(&self, key: u64) -> Option<Arc<SensorStage>> {
        self.sensor_cache.get(key)
    }

    pub(crate) fn put(&self, key: u64, stage: Arc<SensorStage>) {
        self.sensor_cache.put(key, stage);
    }
}

pub(crate) fn sensor_cacheable(options: &RenderOptions) -> bool {
    !options.quality && matches!(options.preview_mode, PreviewMode::None)
}

pub(crate) fn sensor_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    setup: &DcpSetup,
    options: &RenderOptions,
    preview_ratio: Option<f32>,
    superpixel_block: Option<usize>,
    denoise: Option<&DenoisePlane>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    frame.id.hash(&mut h);
    frame.meta.orientation.hash(&mut h);
    frame.meta.is_raw.hash(&mut h);
    for v in frame.meta.wb_coeffs {
        v.to_bits().hash(&mut h);
    }
    frame.meta.capture_sigma.map(f32::to_bits).hash(&mut h);
    for row in setup.cam_to_srgb {
        for v in row {
            v.to_bits().hash(&mut h);
        }
    }
    edits.lens.hash_key(&mut h);
    crate::edits::hash_strokes(&edits.retouch, &mut h);
    let d = &edits.detail;
    d.hash_nr(&mut h);
    d.capture_sharpen.hash(&mut h);
    d.capture_corner_boost.to_bits().hash(&mut h);
    crate::ai_denoise::hash_active(denoise, edits, &mut h);
    preview_ratio.map(f32::to_bits).hash(&mut h);
    superpixel_block.hash(&mut h);
    options.roi.is_some().hash(&mut h);
    if let Some(roi) = options.roi {
        roi.hash_key(&mut h);
    }
    h.finish()
}
