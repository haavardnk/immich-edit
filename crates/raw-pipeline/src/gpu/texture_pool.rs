use std::collections::HashMap;
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

pub struct TexturePool {
    free: Mutex<HashMap<TextureKey, Vec<Arc<Texture>>>>,
    cap_per_key: usize,
}

impl TexturePool {
    pub fn new(cap_per_key: usize) -> Arc<Self> {
        Arc::new(Self {
            free: Mutex::new(HashMap::new()),
            cap_per_key,
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
            g.get_mut(&key).and_then(|v| v.pop())
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
        let mut g = self.free.lock();
        let v = g.entry(key).or_default();
        if v.len() < self.cap_per_key {
            v.push(tex);
        }
    }

    pub fn bytes(&self) -> u64 {
        let g = self.free.lock();
        g.iter()
            .map(|(k, v)| texture_bytes(k) * v.len() as u64)
            .sum()
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
