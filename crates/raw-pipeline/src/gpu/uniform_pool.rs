use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{
    BindingResource, Buffer, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct UniformPool {
    free: Mutex<HashMap<u64, Vec<Arc<Buffer>>>>,
    cap_per_size: usize,
}

impl UniformPool {
    pub fn new(cap_per_size: usize) -> Arc<Self> {
        Arc::new(Self {
            free: Mutex::new(HashMap::new()),
            cap_per_size,
        })
    }

    pub fn acquire(
        self: &Arc<Self>,
        device: &Device,
        queue: &Queue,
        contents: &[u8],
        label: &'static str,
    ) -> PooledUniform {
        let size = contents.len() as u64;
        let from_pool = {
            let mut g = self.free.lock();
            g.get_mut(&size).and_then(|v| v.pop())
        };
        let buffer = match from_pool {
            Some(b) => {
                queue.write_buffer(&b, 0, contents);
                b
            }
            None => Arc::new(device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })),
        };
        PooledUniform {
            buffer: Some(buffer),
            size,
            pool: self.clone(),
        }
    }

    fn release(&self, size: u64, buf: Arc<Buffer>) {
        let mut g = self.free.lock();
        let v = g.entry(size).or_default();
        if v.len() < self.cap_per_size {
            v.push(buf);
        }
    }

    pub fn bytes(&self) -> u64 {
        let g = self.free.lock();
        g.iter().map(|(s, v)| s * v.len() as u64).sum()
    }
}

pub struct PooledUniform {
    buffer: Option<Arc<Buffer>>,
    size: u64,
    pool: Arc<UniformPool>,
}

impl PooledUniform {
    pub fn buffer(&self) -> &Buffer {
        self.buffer.as_ref().expect("pooled uniform taken")
    }

    pub fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer().as_entire_binding()
    }
}

impl Drop for PooledUniform {
    fn drop(&mut self) {
        if let Some(b) = self.buffer.take()
            && Arc::strong_count(&b) == 1
        {
            self.pool.release(self.size, b);
        }
    }
}
