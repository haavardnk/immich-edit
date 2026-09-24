use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use wgpu::{
    Adapter, AdapterInfo, Backends, Device, DeviceDescriptor, DeviceType, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, Queue, TextureFormat, TextureFormatFeatureFlags,
    TextureUsages,
};

use super::timer::TimerSlots;
use crate::{PipelineError, PipelineResult};

pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub adapter_info: AdapterInfo,
    pub linear_format: TextureFormat,
    pub timestamps: bool,
    pub(crate) timer_slots: TimerSlots,
    device_lost: Arc<AtomicBool>,
}

impl GpuContext {
    #[cfg(feature = "native")]
    pub fn new() -> PipelineResult<Arc<Self>> {
        Self::with_timestamps(false)
    }

    #[cfg(feature = "native")]
    pub fn with_timestamps(timestamps: bool) -> PipelineResult<Arc<Self>> {
        pollster::block_on(Self::new_async(timestamps))
    }

    pub async fn new_async(timestamps: bool) -> PipelineResult<Arc<Self>> {
        let mut instance_desc = InstanceDescriptor::new_without_display_handle();
        instance_desc.backends = Backends::PRIMARY;
        let instance = Instance::new(instance_desc);
        let adapters = instance.enumerate_adapters(Backends::PRIMARY).await;
        let infos: Vec<AdapterInfo> = adapters.iter().map(|a| a.get_info()).collect();
        let types: Vec<DeviceType> = infos.iter().map(|i| i.device_type).collect();
        let index = pick_adapter_index(&types)
            .ok_or_else(|| PipelineError::Unsupported("gpu adapter: none available".into()))?;
        let adapter = adapters
            .into_iter()
            .nth(index)
            .expect("index comes from the same list");

        let adapter_info = infos
            .into_iter()
            .nth(index)
            .expect("index comes from the same list");
        let adapter_limits = adapter.limits();
        let linear_format = pick_linear_format(&adapter);

        let mut limits = Limits::default();
        limits.max_texture_dimension_2d = limits
            .max_texture_dimension_2d
            .max(8192)
            .min(adapter_limits.max_texture_dimension_2d);
        limits.max_storage_buffer_binding_size = limits
            .max_storage_buffer_binding_size
            .max(256 * 1024 * 1024)
            .min(adapter_limits.max_storage_buffer_binding_size);
        limits.max_buffer_size = limits
            .max_buffer_size
            .max(512 * 1024 * 1024)
            .min(adapter_limits.max_buffer_size);

        let timestamps = timestamps && adapter.features().contains(Features::TIMESTAMP_QUERY);
        let required_features = if timestamps {
            Features::TIMESTAMP_QUERY
        } else {
            Features::empty()
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("immich-edit gpu"),
                required_features,
                required_limits: limits,
                experimental_features: Default::default(),
                memory_hints: MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| PipelineError::Unsupported(format!("gpu device: {e}")))?;

        let device_lost = Arc::new(AtomicBool::new(false));
        let flag = device_lost.clone();
        device.set_device_lost_callback(move |reason, msg| {
            tracing::error!(?reason, msg = %msg, "gpu device lost");
            flag.store(true, Ordering::SeqCst);
        });

        Ok(Arc::new(Self {
            instance,
            adapter,
            device,
            queue,
            adapter_info,
            linear_format,
            timestamps,
            timer_slots: TimerSlots::default(),
            device_lost,
        }))
    }

    pub fn is_lost(&self) -> bool {
        self.device_lost.load(Ordering::SeqCst)
    }

    pub fn adapter_label(&self) -> String {
        adapter_label(&self.adapter_info)
    }

    pub fn is_software(&self) -> bool {
        self.adapter_info.device_type == DeviceType::Cpu
    }
}

pub fn adapter_label(info: &AdapterInfo) -> String {
    format!("{} ({:?}, {:?})", info.name, info.device_type, info.backend)
}

fn pick_adapter_index(types: &[DeviceType]) -> Option<usize> {
    types
        .iter()
        .enumerate()
        .min_by_key(|(_, t)| adapter_rank(**t))
        .map(|(i, _)| i)
}

fn adapter_rank(device_type: DeviceType) -> u8 {
    match device_type {
        DeviceType::DiscreteGpu => 0,
        DeviceType::IntegratedGpu => 1,
        DeviceType::VirtualGpu => 2,
        DeviceType::Other => 3,
        DeviceType::Cpu => 4,
    }
}

fn pick_linear_format(adapter: &Adapter) -> TextureFormat {
    let prefer = TextureFormat::Rgba16Float;
    let feats = adapter.get_texture_format_features(prefer);
    let needs_usage = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let needs_flags = TextureFormatFeatureFlags::FILTERABLE;
    if feats.allowed_usages.contains(needs_usage) && feats.flags.contains(needs_flags) {
        return prefer;
    }
    TextureFormat::Rgba32Float
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_wins_over_integrated_and_software() {
        let types = [
            DeviceType::Cpu,
            DeviceType::IntegratedGpu,
            DeviceType::DiscreteGpu,
        ];
        assert_eq!(pick_adapter_index(&types), Some(2));
    }

    #[test]
    fn software_is_the_last_resort_but_still_chosen() {
        assert_eq!(pick_adapter_index(&[DeviceType::Cpu]), Some(0));
        assert_eq!(
            pick_adapter_index(&[DeviceType::Cpu, DeviceType::Other]),
            Some(1)
        );
    }

    #[test]
    fn no_adapters_picks_nothing() {
        assert_eq!(pick_adapter_index(&[]), None);
    }

    #[test]
    fn the_first_adapter_of_the_best_rank_wins() {
        let types = [
            DeviceType::Other,
            DeviceType::IntegratedGpu,
            DeviceType::IntegratedGpu,
        ];
        assert_eq!(pick_adapter_index(&types), Some(1));
    }
}
