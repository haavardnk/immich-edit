use std::sync::Arc;

use lru::LruCache;
use parking_lot::Mutex;
use wgpu::Texture;

use super::pools::texture_bytes;
use crate::gpu::budget::GpuBudget;
use crate::gpu::texture_pool::TexturePool;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub(in crate::gpu::renderer) enum Stage {
    Wb,
    Nr,
    Capture,
}

struct Entry {
    texture: Arc<Texture>,
    bytes: u64,
}

pub(in crate::gpu::renderer) struct StageCache {
    entries: Mutex<LruCache<(Stage, u64), Entry>>,
    budget: Arc<GpuBudget>,
    pool: Arc<TexturePool>,
}

impl StageCache {
    pub fn new(budget: Arc<GpuBudget>, pool: Arc<TexturePool>) -> Self {
        Self {
            entries: Mutex::new(LruCache::unbounded()),
            budget,
            pool,
        }
    }

    pub fn get(&self, stage: Stage, key: u64) -> Option<Arc<Texture>> {
        self.entries
            .lock()
            .get(&(stage, key))
            .map(|e| e.texture.clone())
    }

    pub fn put(&self, stage: Stage, key: u64, texture: Arc<Texture>) {
        let bytes = texture_bytes(&texture);
        if bytes > self.budget.max_bytes() {
            return;
        }
        let mut entries = self.entries.lock();
        if let Some(old) = entries.pop(&(stage, key)) {
            self.budget.release(old.bytes);
        }
        while !self.budget.try_reserve(bytes) {
            if self.pool.trim(bytes) > 0 {
                continue;
            }
            let Some((_, evicted)) = entries.pop_lru() else {
                return;
            };
            self.budget.release(evicted.bytes);
        }
        entries.put((stage, key), Entry { texture, bytes });
    }

    pub fn bytes(&self, stage: Stage) -> u64 {
        self.entries
            .lock()
            .iter()
            .filter(|((s, _), _)| *s == stage)
            .map(|(_, e)| e.bytes)
            .sum()
    }
}
