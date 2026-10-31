use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use parking_lot::Mutex;

use crate::admission::{Retained, evict_idle, idle_bytes, unshared};

const SENSOR_CACHE_ITEMS: usize = 2;

pub(crate) struct SensorStage {
    pub rgb: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub oriented_w: usize,
    pub oriented_h: usize,
}

fn stage_bytes(stage: &Arc<SensorStage>) -> u64 {
    std::mem::size_of_val(stage.rgb.as_slice()) as u64
}

pub(super) struct SensorCache(Mutex<LruCache<u64, Arc<SensorStage>>>);

impl Default for SensorCache {
    fn default() -> Self {
        Self(Mutex::new(LruCache::new(
            NonZeroUsize::new(SENSOR_CACHE_ITEMS).unwrap(),
        )))
    }
}

impl SensorCache {
    pub fn get(&self, key: u64) -> Option<Arc<SensorStage>> {
        self.0.lock().get(&key).cloned()
    }

    pub fn put(&self, key: u64, stage: Arc<SensorStage>) {
        self.0.lock().put(key, stage);
    }
}

impl Retained for SensorCache {
    fn retained_bytes(&self) -> u64 {
        idle_bytes(&self.0.lock(), unshared, stage_bytes)
    }

    fn trim(&self, excess: u64) {
        evict_idle(&mut self.0.lock(), unshared, stage_bytes, excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(len: usize) -> Arc<SensorStage> {
        Arc::new(SensorStage {
            rgb: vec![0.0; len],
            width: len / 3,
            height: 1,
            oriented_w: len / 3,
            oriented_h: 1,
        })
    }

    #[test]
    fn only_idle_stages_are_retained_and_trimmed() {
        let cache = SensorCache::default();
        let pinned = stage(300);
        cache.put(1, pinned.clone());
        cache.put(2, stage(600));
        if cache.retained_bytes() != 2400 {
            panic!(
                "retained {} bytes, expected only the idle stage",
                cache.retained_bytes()
            );
        }
        cache.trim(u64::MAX);
        if cache.get(1).is_none() || cache.get(2).is_some() {
            panic!("trimming evicted a stage in use or kept an idle one");
        }
    }
}
