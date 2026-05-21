use std::sync::Arc;

use wgpu::{
    Adapter, AdapterInfo, Backends, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
    TextureFormat, TextureFormatFeatureFlags, TextureUsages,
};

use crate::{PipelineError, PipelineResult};

pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub adapter_info: AdapterInfo,
    pub linear_format: TextureFormat,
}

impl GpuContext {
    pub fn new() -> PipelineResult<Arc<Self>> {
        pollster::block_on(Self::new_async())
    }

    pub async fn new_async() -> PipelineResult<Arc<Self>> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| PipelineError::Unsupported("no gpu adapter found".into()))?;

        let adapter_info = adapter.get_info();
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

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("immich-edit gpu"),
                    required_features: Features::empty(),
                    required_limits: limits,
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| PipelineError::Unsupported(format!("gpu device: {e}")))?;

        Ok(Arc::new(Self {
            instance,
            adapter,
            device,
            queue,
            adapter_info,
            linear_format,
        }))
    }

    pub fn adapter_label(&self) -> String {
        format!(
            "{} ({:?}, {:?})",
            self.adapter_info.name, self.adapter_info.device_type, self.adapter_info.backend
        )
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
