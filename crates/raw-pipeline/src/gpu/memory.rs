use serde::Serialize;
use wgpu::Device;

#[cfg(all(feature = "native", target_vendor = "apple"))]
mod metal;
#[cfg(all(feature = "native", not(target_vendor = "apple")))]
mod vulkan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeviceMemory {
    pub bytes: u64,
    pub unified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GpuMemory {
    pub device: Option<DeviceMemory>,
    pub host_budget_bytes: Option<u64>,
    pub budget_bytes: Option<u64>,
    pub texture_cache_bytes: u64,
}

#[cfg(all(test, feature = "native", target_vendor = "apple"))]
pub(crate) use metal::allocated_bytes;

type Probe = fn(&Device) -> Option<DeviceMemory>;

const PROBES: &[Probe] = &[
    #[cfg(all(feature = "native", target_vendor = "apple"))]
    metal::probe,
    #[cfg(all(feature = "native", not(target_vendor = "apple")))]
    vulkan::probe,
];

pub(super) fn probe(device: &Device) -> Option<DeviceMemory> {
    PROBES.iter().find_map(|probe| probe(device))
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use wgpu::Backend;

    use crate::gpu::context::GpuContext;

    #[test]
    fn metal_and_vulkan_devices_report_their_memory() {
        let Ok(ctx) = GpuContext::new() else {
            return;
        };
        let probed = matches!(ctx.adapter_info.backend, Backend::Metal | Backend::Vulkan);
        if probed != ctx.memory.is_some_and(|memory| memory.bytes > 0) {
            panic!("{} reported {:?}", ctx.adapter_label(), ctx.memory);
        }
    }
}
