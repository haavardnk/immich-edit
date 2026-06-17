use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

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
}

struct FreePool {
    by_key: HashMap<TextureKey, Vec<Arc<Texture>>>,
    lru: VecDeque<TextureKey>,
    retained_bytes: u64,
}

pub struct TexturePool {
    free: Mutex<FreePool>,
    cap_per_key: usize,
    max_bytes: u64,
}

impl TexturePool {
    pub fn new(cap_per_key: usize, max_bytes: u64) -> Arc<Self> {
        Arc::new(Self {
            free: Mutex::new(FreePool {
                by_key: HashMap::new(),
                lru: VecDeque::new(),
                retained_bytes: 0,
            }),
            cap_per_key,
            max_bytes,
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
                g.retained_bytes = g.retained_bytes.saturating_sub(texture_bytes(&key));
            }
            popped
        };
        let tex = from_pool.unwrap_or_else(|| {
            Arc::new(device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: key.width,
                    height: key.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: key.mip_level_count,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: key.format,
                usage: TextureUsages::from_bits_truncate(key.usage_bits),
                view_formats: &[],
            }))
        });
        PooledTexture {
            texture: Some(tex),
            key,
            pool: self.clone(),
        }
    }

    fn release(&self, key: TextureKey, tex: Arc<Texture>) {
        let tex_bytes = texture_bytes(&key);
        let mut g = self.free.lock();
        if g.by_key.get(&key).map(Vec::len).unwrap_or(0) >= self.cap_per_key {
            return;
        }
        if tex_bytes > self.max_bytes {
            return;
        }
        while g.retained_bytes + tex_bytes > self.max_bytes {
            let Some(victim) = g.lru.pop_front() else {
                break;
            };
            if let Some(v) = g.by_key.get_mut(&victim)
                && v.pop().is_some()
            {
                g.retained_bytes = g.retained_bytes.saturating_sub(texture_bytes(&victim));
            }
        }
        if g.retained_bytes + tex_bytes > self.max_bytes {
            return;
        }
        g.by_key.entry(key).or_default().push(tex);
        g.lru.push_back(key);
        g.retained_bytes += tex_bytes;
    }

    pub fn bytes(&self) -> u64 {
        self.free.lock().retained_bytes
    }
}

fn texture_bytes(k: &TextureKey) -> u64 {
    let bpp = k.format.block_copy_size(None).unwrap_or(0) as u64;
    let w = k.width as u64;
    let h = k.height as u64;
    let mut total: u64 = 0;
    for level in 0..k.mip_level_count {
        let lw = (w >> level).max(1);
        let lh = (h >> level).max(1);
        total += lw * lh * bpp;
    }
    total
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

    pub fn into_arc(mut self) -> Arc<Texture> {
        self.texture.take().expect("pooled texture taken")
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
