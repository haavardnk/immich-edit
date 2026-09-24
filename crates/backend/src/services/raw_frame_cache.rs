use std::mem::size_of;
use std::sync::Arc;

use raw_pipeline::frame::RawFrame;
use uuid::Uuid;

const FRAME_OVERHEAD_BYTES: u64 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameCacheKey {
    pub server_epoch: i64,
    pub owner: Uuid,
    pub asset_id: Uuid,
}

pub struct RawFrameCache {
    map: lru::LruCache<FrameCacheKey, Arc<RawFrame>>,
    max_bytes: u64,
    current_bytes: u64,
}

fn frame_bytes(frame: &RawFrame) -> u64 {
    let data = (frame.data.len() as u64).saturating_mul(size_of::<f32>() as u64);
    data.saturating_add(FRAME_OVERHEAD_BYTES)
}

impl RawFrameCache {
    pub fn new(max_bytes: u64) -> Self {
        Self {
            map: lru::LruCache::unbounded(),
            max_bytes: max_bytes.max(1),
            current_bytes: 0,
        }
    }

    pub fn get(&mut self, key: &FrameCacheKey) -> Option<Arc<RawFrame>> {
        self.map.get(key).cloned()
    }

    pub fn put(&mut self, key: FrameCacheKey, frame: Arc<RawFrame>) {
        let bytes = frame_bytes(&frame);
        if let Some(old) = self.map.pop(&key) {
            self.current_bytes = self.current_bytes.saturating_sub(frame_bytes(&old));
        }
        if bytes > self.max_bytes {
            return;
        }
        while self.current_bytes + bytes > self.max_bytes {
            let Some((_, evicted)) = self.map.pop_lru() else {
                break;
            };
            self.current_bytes = self.current_bytes.saturating_sub(frame_bytes(&evicted));
        }
        if self.current_bytes + bytes > self.max_bytes {
            return;
        }
        self.map.put(key, frame);
        self.current_bytes += bytes;
    }

    pub fn current_bytes(&self) -> u64 {
        self.current_bytes
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.current_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_pipeline::frame::FrameMeta;

    fn frame_with_floats(n: usize) -> Arc<RawFrame> {
        Arc::new(RawFrame {
            meta: FrameMeta {
                width: 1,
                height: n,
                wb_coeffs: [1.0; 4],
                xyz_to_cam: [[0.0; 3]; 4],
                color_matrices: Vec::new(),
                orientation: (false, false, false),
                is_raw: true,
                capture_sigma: None,
                model: String::new(),
            },
            cfa_pattern: String::new(),
            bps: 16,
            data: vec![0.0f32; n],
            cpp: 1,
            exif: None,
        })
    }

    fn mb(n: u64) -> u64 {
        n * 1024 * 1024
    }

    fn key(owner: Uuid, asset_id: Uuid) -> FrameCacheKey {
        FrameCacheKey {
            server_epoch: 1,
            owner,
            asset_id,
        }
    }

    #[test]
    fn evicts_lru_to_fit_budget() {
        let floats = (mb(1) / 4) as usize;
        let mut cache = RawFrameCache::new(mb(2) + FRAME_OVERHEAD_BYTES * 2);
        let owner = Uuid::new_v4();
        let a = key(owner, Uuid::new_v4());
        let b = key(owner, Uuid::new_v4());
        let c = key(owner, Uuid::new_v4());
        cache.put(a, frame_with_floats(floats));
        cache.put(b, frame_with_floats(floats));
        cache.get(&a);
        cache.put(c, frame_with_floats(floats));
        if cache.get(&b).is_some() {
            panic!("expected LRU entry b evicted");
        }
        if cache.get(&a).is_none() || cache.get(&c).is_none() {
            panic!("expected a and c retained");
        }
    }

    #[test]
    fn owners_do_not_share_frames() {
        let mut cache = RawFrameCache::new(mb(4));
        let asset = Uuid::new_v4();
        let a = key(Uuid::new_v4(), asset);
        let b = key(Uuid::new_v4(), asset);
        cache.put(a, frame_with_floats(16));
        if cache.get(&b).is_some() {
            panic!("another owner must not hit the cached frame");
        }
        if cache.get(&a).is_none() {
            panic!("owner a should still hit");
        }
    }

    #[test]
    fn skips_oversized_frame() {
        let floats = (mb(4) / 4) as usize;
        let mut cache = RawFrameCache::new(mb(1));
        let id = key(Uuid::new_v4(), Uuid::new_v4());
        cache.put(id, frame_with_floats(floats));
        if cache.get(&id).is_some() {
            panic!("oversized frame should not be retained");
        }
        if cache.current_bytes() != 0 {
            panic!("current_bytes should stay 0");
        }
    }
}
