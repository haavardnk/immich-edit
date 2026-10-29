use ash::vk;
use wgpu::Device;
use wgpu::hal::api::Vulkan;

use super::DeviceMemory;

pub(super) fn probe(device: &Device) -> Option<DeviceMemory> {
    let hal = unsafe { device.as_hal::<Vulkan>() }?;
    let shared = hal.shared_instance();
    let instance = shared.raw_instance();
    let physical = hal.raw_physical_device();
    let kind = unsafe { instance.get_physical_device_properties(physical) }.device_type;
    let budgeted = shared.instance_api_version() >= vk::API_VERSION_1_1
        && hal
            .enabled_device_extensions()
            .contains(&ash::ext::memory_budget::NAME);
    let bytes = if budgeted {
        let mut budget = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut properties = vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
        unsafe { instance.get_physical_device_memory_properties2(physical, &mut properties) };
        let heaps = properties.memory_properties;
        device_local_bytes(&heaps, Some(&budget.heap_budget))
    } else {
        let heaps = unsafe { instance.get_physical_device_memory_properties(physical) };
        device_local_bytes(&heaps, None)
    }?;
    Some(DeviceMemory {
        bytes,
        unified: matches!(
            kind,
            vk::PhysicalDeviceType::INTEGRATED_GPU | vk::PhysicalDeviceType::CPU
        ),
    })
}

fn device_local_bytes(
    properties: &vk::PhysicalDeviceMemoryProperties,
    budgets: Option<&[u64]>,
) -> Option<u64> {
    let (index, heap) = properties
        .memory_heaps_as_slice()
        .iter()
        .enumerate()
        .filter(|(_, heap)| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
        .max_by_key(|(_, heap)| heap.size)?;
    let budget = budgets
        .and_then(|budgets| budgets.get(index).copied())
        .filter(|budget| *budget > 0);
    Some(budget.map_or(heap.size, |budget| budget.min(heap.size)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heaps(entries: &[(u64, vk::MemoryHeapFlags)]) -> vk::PhysicalDeviceMemoryProperties {
        let mut properties = vk::PhysicalDeviceMemoryProperties {
            memory_heap_count: entries.len() as u32,
            ..Default::default()
        };
        for (slot, (size, flags)) in properties.memory_heaps.iter_mut().zip(entries) {
            *slot = vk::MemoryHeap {
                size: *size,
                flags: *flags,
            };
        }
        properties
    }

    #[test]
    fn the_largest_device_local_heap_sets_the_bytes() {
        let properties = heaps(&[
            (256 << 20, vk::MemoryHeapFlags::DEVICE_LOCAL),
            (32 << 30, vk::MemoryHeapFlags::empty()),
            (8 << 30, vk::MemoryHeapFlags::DEVICE_LOCAL),
        ]);
        if device_local_bytes(&properties, None) != Some(8 << 30) {
            panic!("the 8 GiB device-local heap was not chosen");
        }
    }

    #[test]
    fn the_budget_of_that_heap_wins_when_reported() {
        let properties = heaps(&[
            (256 << 20, vk::MemoryHeapFlags::DEVICE_LOCAL),
            (8 << 30, vk::MemoryHeapFlags::DEVICE_LOCAL),
        ]);
        let mut budgets = [0; vk::MAX_MEMORY_HEAPS];
        budgets[0] = 200 << 20;
        budgets[1] = 6 << 30;
        if device_local_bytes(&properties, Some(&budgets)) != Some(6 << 30) {
            panic!("the reported 6 GiB budget was not used");
        }
        budgets[1] = 0;
        if device_local_bytes(&properties, Some(&budgets)) != Some(8 << 30) {
            panic!("an unreported budget did not fall back to the heap size");
        }
    }

    #[test]
    fn no_device_local_heap_reports_nothing() {
        let properties = heaps(&[(16 << 30, vk::MemoryHeapFlags::empty())]);
        if device_local_bytes(&properties, None).is_some() {
            panic!("a host-only heap was reported as device memory");
        }
    }
}
