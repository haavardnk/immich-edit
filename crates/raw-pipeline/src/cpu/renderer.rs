use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use parking_lot::Mutex;

use crate::cancel::CancelToken;
use crate::dcp_pipeline::DcpSetup;
use crate::edits::Edits;
use crate::frame::{PreviewMode, RawFrame, RenderOptions, RenderedImage};

const SENSOR_CACHE_ITEMS: usize = 2;

pub(crate) struct SensorStage {
    pub rgb: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub oriented_w: usize,
    pub oriented_h: usize,
}

pub struct CpuRenderer {
    sensor_cache: Mutex<LruCache<u64, Arc<SensorStage>>>,
}

impl Default for CpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self {
            sensor_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(SENSOR_CACHE_ITEMS).unwrap(),
            )),
        }
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

    pub(crate) fn get(&self, key: u64) -> Option<Arc<SensorStage>> {
        self.sensor_cache.lock().get(&key).cloned()
    }

    pub(crate) fn put(&self, key: u64, stage: Arc<SensorStage>) {
        self.sensor_cache.lock().put(key, stage);
    }
}

pub(crate) fn frame_cache_key(frame: &RawFrame) -> u64 {
    let ptr = frame.data.as_ptr() as usize as u64;
    let dims = ((frame.width as u64) << 32) | (frame.height as u64);
    ptr ^ dims
}

pub(crate) fn sensor_cacheable(edits: &Edits, options: &RenderOptions) -> bool {
    if options.quality {
        return false;
    }
    if !matches!(options.preview_mode, PreviewMode::None) {
        return false;
    }
    !edits
        .masks
        .iter()
        .any(|l| l.is_effective() && (l.edits.wb_temp.is_some() || l.edits.wb_tint.is_some()))
}

pub(crate) fn sensor_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    setup: &DcpSetup,
    options: &RenderOptions,
    preview_ratio: Option<f32>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    frame_cache_key(frame).hash(&mut h);
    frame.orientation.hash(&mut h);
    frame.is_raw.hash(&mut h);
    for v in frame.wb_coeffs {
        v.to_bits().hash(&mut h);
    }
    frame.capture_sigma.map(f32::to_bits).hash(&mut h);
    for row in setup.cam_to_srgb {
        for v in row {
            v.to_bits().hash(&mut h);
        }
    }
    setup
        .resolved
        .as_ref()
        .map(|d| Arc::as_ptr(d) as usize)
        .hash(&mut h);
    edits.basic.wb_temp.to_bits().hash(&mut h);
    edits.basic.wb_tint.to_bits().hash(&mut h);
    serde_json::to_vec(&edits.lens)
        .unwrap_or_default()
        .hash(&mut h);
    serde_json::to_vec(&edits.retouch)
        .unwrap_or_default()
        .hash(&mut h);
    let d = &edits.detail;
    d.luma_nr_amount.to_bits().hash(&mut h);
    d.luma_nr_detail.to_bits().hash(&mut h);
    d.luma_nr_contrast.to_bits().hash(&mut h);
    d.color_nr_amount.to_bits().hash(&mut h);
    d.color_nr_detail.to_bits().hash(&mut h);
    d.color_nr_smoothness.to_bits().hash(&mut h);
    d.capture_sharpen.hash(&mut h);
    preview_ratio.map(f32::to_bits).hash(&mut h);
    serde_json::to_vec(&options.roi)
        .unwrap_or_default()
        .hash(&mut h);
    h.finish()
}
