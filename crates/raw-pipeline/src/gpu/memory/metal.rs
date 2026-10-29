use objc2_metal::MTLDevice;
use wgpu::Device;
use wgpu::hal::api::Metal;

use super::DeviceMemory;

pub(super) fn probe(device: &Device) -> Option<DeviceMemory> {
    let hal = unsafe { device.as_hal::<Metal>() }?;
    let raw = hal.raw_device();
    Some(DeviceMemory {
        bytes: raw.recommendedMaxWorkingSetSize(),
        unified: raw.hasUnifiedMemory(),
    })
}

#[cfg(test)]
pub(crate) fn allocated_bytes(device: &Device) -> Option<u64> {
    let hal = unsafe { device.as_hal::<Metal>() }?;
    Some(hal.raw_device().currentAllocatedSize() as u64)
}
