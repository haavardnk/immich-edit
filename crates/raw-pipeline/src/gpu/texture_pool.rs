use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{Device, Texture, TextureFormat, TextureUsages};

use super::budget::GpuBudget;
use super::texture::{texture_2d, texture_size};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct TextureKey {
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub mip_level_count: u32,
    pub usage_bits: u32,
}

impl TextureKey {
    pub fn new(
        format: TextureFormat,
        width: u32,
        height: u32,
        mip_level_count: u32,
        usage: TextureUsages,
    ) -> Self {
        Self {
            format,
            width,
            height,
            mip_level_count,
            usage_bits: usage.bits(),
        }
    }

    fn bytes(&self) -> u64 {
        texture_size(self.format, (self.width, self.height), self.mip_level_count)
    }
}

struct FreePool {
    by_key: HashMap<TextureKey, Vec<Arc<Texture>>>,
    lru: VecDeque<TextureKey>,
    retained_bytes: u64,
}

pub struct TexturePool {
    free: Mutex<FreePool>,
    cap_per_key: usize,
    budget: Arc<GpuBudget>,
}

impl TexturePool {
    pub fn new(cap_per_key: usize, budget: Arc<GpuBudget>) -> Arc<Self> {
        Arc::new(Self {
            free: Mutex::new(FreePool {
                by_key: HashMap::new(),
                lru: VecDeque::new(),
                retained_bytes: 0,
            }),
            cap_per_key,
            budget,
        })
    }

    pub fn acquire(
        self: &Arc<Self>,
        device: &Device,
        key: TextureKey,
        label: &'static str,
    ) -> PooledTexture {
        let from_pool = {
            let mut g = self.free.lock();
            let popped = g.by_key.get_mut(&key).and_then(|v| v.pop());
            if popped.is_some() {
                if let Some(pos) = g.lru.iter().position(|k| *k == key) {
                    g.lru.remove(pos);
                }
                let bytes = key.bytes();
                g.retained_bytes = g.retained_bytes.saturating_sub(bytes);
                self.budget.release(bytes);
            }
            popped
        };
        let tex = from_pool.unwrap_or_else(|| {
            Arc::new(texture_2d(
                device,
                label,
                key.format,
                (key.width, key.height),
                key.mip_level_count,
                TextureUsages::from_bits_truncate(key.usage_bits),
            ))
        });
        PooledTexture {
            texture: Some(tex),
            key,
            pool: self.clone(),
        }
    }

    fn release(&self, key: TextureKey, tex: Arc<Texture>) {
        let tex_bytes = key.bytes();
        let mut g = self.free.lock();
        if g.by_key.get(&key).map(Vec::len).unwrap_or(0) >= self.cap_per_key {
            return;
        }
        while !self.budget.try_reserve(tex_bytes) {
            if !evict_one(&mut g, &self.budget) {
                return;
            }
        }
        g.by_key.entry(key).or_default().push(tex);
        g.lru.push_back(key);
        g.retained_bytes += tex_bytes;
    }

    pub fn trim(&self, bytes: u64) -> u64 {
        let mut g = self.free.lock();
        let start = g.retained_bytes;
        while start - g.retained_bytes < bytes && evict_one(&mut g, &self.budget) {}
        start - g.retained_bytes
    }

    pub fn bytes(&self) -> u64 {
        self.free.lock().retained_bytes
    }
}

fn evict_one(g: &mut FreePool, budget: &GpuBudget) -> bool {
    let Some(victim) = g.lru.pop_front() else {
        return false;
    };
    if let Some(v) = g.by_key.get_mut(&victim)
        && v.pop().is_some()
    {
        let bytes = victim.bytes();
        g.retained_bytes = g.retained_bytes.saturating_sub(bytes);
        budget.release(bytes);
    }
    true
}

pub struct PooledTexture {
    texture: Option<Arc<Texture>>,
    key: TextureKey,
    pool: Arc<TexturePool>,
}

impl PooledTexture {
    pub fn texture(&self) -> &Texture {
        self.texture.as_ref().expect("pooled texture taken")
    }

    pub fn shared(&self) -> Arc<Texture> {
        self.texture.clone().expect("pooled texture taken")
    }
}

impl std::ops::Deref for PooledTexture {
    type Target = Texture;
    fn deref(&self) -> &Texture {
        self.texture()
    }
}

impl Drop for PooledTexture {
    fn drop(&mut self) {
        if let Some(t) = self.texture.take()
            && Arc::strong_count(&t) == 1
        {
            self.pool.release(self.key, t);
        }
    }
}
