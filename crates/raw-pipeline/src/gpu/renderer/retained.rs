use std::sync::Arc;

use parking_lot::Mutex;

use super::GpuRenderer;
use super::pools::{PoolTarget, texture_bytes};
use super::sensor_stage::CachedFrame;

type FrameCache = Mutex<lru::LruCache<u64, Arc<CachedFrame>>>;

impl GpuRenderer {
    pub(super) fn retained_bytes(&self) -> u64 {
        self.texture_pool.bytes()
            + self.sensor.stages.idle_bytes()
            + idle_targets_bytes(&self.output_pool)
            + idle_targets_bytes(&self.sharpen_pool)
            + idle_frames_bytes(&self.sensor.superpixels)
            + idle_frames_bytes(&self.sensor.frames)
    }

    pub(super) fn trim_retained(&self, excess: u64) {
        let target = self.retained_bytes().saturating_sub(excess);
        let steps: [&dyn Fn(u64); 4] = [
            &|bytes| {
                self.texture_pool.trim(bytes);
            },
            &|bytes| self.sensor.stages.trim(bytes),
            &|_| {
                clear_idle_targets(&self.output_pool);
                clear_idle_targets(&self.sharpen_pool);
            },
            &|_| {
                evict_idle_frames(&self.sensor.superpixels);
                evict_idle_frames(&self.sensor.frames);
            },
        ];
        for step in steps {
            let retained = self.retained_bytes();
            if retained <= target {
                return;
            }
            step(retained - target);
        }
    }
}

fn idle_targets_bytes<T: PoolTarget>(pool: &Mutex<Vec<T>>) -> u64 {
    pool.try_lock()
        .map_or(0, |targets| targets.iter().map(T::bytes).sum())
}

fn clear_idle_targets<T>(pool: &Mutex<Vec<T>>) {
    if let Some(mut targets) = pool.try_lock() {
        targets.clear();
    }
}

fn is_idle(frame: &Arc<CachedFrame>) -> bool {
    Arc::strong_count(frame) == 1 && Arc::strong_count(&frame.texture) == 1
}

fn idle_frames_bytes(frames: &FrameCache) -> u64 {
    frames
        .lock()
        .iter()
        .filter(|(_, frame)| is_idle(frame))
        .map(|(_, frame)| texture_bytes(&frame.texture))
        .sum()
}

fn evict_idle_frames(frames: &FrameCache) {
    let mut frames = frames.lock();
    let idle: Vec<u64> = frames
        .iter()
        .filter(|(_, frame)| is_idle(frame))
        .map(|(key, _)| *key)
        .collect();
    for key in idle {
        frames.pop(&key);
    }
}
