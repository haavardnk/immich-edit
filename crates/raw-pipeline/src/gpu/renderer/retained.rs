use std::sync::Arc;

use parking_lot::Mutex;

use super::pools::{PoolTarget, texture_bytes};
use super::sensor_stage::{SensorCaches, SensorFrame};
use super::{GpuRenderer, WorkingBytes};
use crate::PipelineResult;
use crate::admission::{Admitted, Retained, evict_idle, idle_bytes, unshared};
use crate::cancel::CancelToken;
use crate::gpu::resources::{OutputTargets, SharpenTargets};
use crate::gpu::texture_pool::TexturePool;
use crate::source::ContextPlanes;

pub(super) struct GpuRetention {
    pub texture_pool: Arc<TexturePool>,
    pub sensor: Arc<SensorCaches>,
    pub output_pool: Arc<Mutex<Vec<OutputTargets>>>,
    pub sharpen_pool: Arc<Mutex<Vec<SharpenTargets>>>,
    pub unified: bool,
}

impl GpuRetention {
    pub fn device_bytes(&self) -> u64 {
        self.texture_pool.bytes()
            + self.sensor.stages.idle_bytes()
            + idle_targets_bytes(&self.output_pool)
            + idle_targets_bytes(&self.sharpen_pool)
            + idle_bytes(&self.sensor.superpixels.lock(), frame_idle, frame_bytes)
            + idle_bytes(&self.sensor.frames.lock(), frame_idle, frame_bytes)
    }

    pub fn trim_device(&self, excess: u64) {
        let target = self.device_bytes().saturating_sub(excess);
        let steps: [&dyn Fn(u64); 4] = [
            &|bytes| {
                self.texture_pool.trim(bytes);
            },
            &|bytes| self.sensor.stages.trim(bytes),
            &|_| {
                clear_idle_targets(&self.output_pool);
                clear_idle_targets(&self.sharpen_pool);
            },
            &|bytes| {
                let mut superpixels = self.sensor.superpixels.lock();
                let freed = evict_idle(&mut superpixels, frame_idle, frame_bytes, bytes);
                let mut frames = self.sensor.frames.lock();
                evict_idle(
                    &mut frames,
                    frame_idle,
                    frame_bytes,
                    bytes - freed.min(bytes),
                );
            },
        ];
        for step in steps {
            let retained = self.device_bytes();
            if retained <= target {
                return;
            }
            step(retained - target);
        }
    }
}

struct DeviceRetained<'a>(&'a GpuRetention);

impl Retained for DeviceRetained<'_> {
    fn retained_bytes(&self) -> u64 {
        self.0.device_bytes()
    }

    fn trim(&self, excess: u64) {
        self.0.trim_device(excess);
    }
}

impl Retained for GpuRetention {
    fn retained_bytes(&self) -> u64 {
        let device = if self.unified { self.device_bytes() } else { 0 };
        device + idle_bytes(&self.sensor.contexts.lock(), unshared, context_bytes)
    }

    fn trim(&self, excess: u64) {
        let target = self.retained_bytes().saturating_sub(excess);
        if self.unified {
            self.trim_device(excess);
        }
        let over = self.retained_bytes().saturating_sub(target);
        evict_idle(
            &mut self.sensor.contexts.lock(),
            unshared,
            context_bytes,
            over,
        );
    }
}

impl GpuRenderer {
    pub(super) fn admit(
        &self,
        bytes: u64,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        let retained = DeviceRetained(&self.retention);
        self.admission.admit(bytes, &retained, cancel)
    }

    pub(super) fn admit_host(
        &self,
        bytes: WorkingBytes,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        self.host.admit(self.host_bytes(bytes), cancel)
    }

    pub(super) fn host_bytes(&self, bytes: WorkingBytes) -> u64 {
        if self.retention.unified {
            bytes.host + bytes.device
        } else {
            bytes.host
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

fn frame_idle(frame: &Arc<SensorFrame>) -> bool {
    unshared(frame) && unshared(&frame.texture)
}

fn frame_bytes(frame: &Arc<SensorFrame>) -> u64 {
    texture_bytes(&frame.texture)
}

fn context_bytes(planes: &Arc<ContextPlanes>) -> u64 {
    planes.bytes()
}
