#[cfg(feature = "native")]
use crate::admission::HostMemory;
use crate::gpu::context::RESOURCE_CREATION_BUDGET_PERCENT;
use crate::gpu::memory::DeviceMemory;
#[cfg(feature = "native")]
use crate::tile::TilePlan;

const MB: u64 = 1024 * 1024;
const DEFAULT_TEXTURE_CACHE_BYTES: u64 = 512 * MB;
const MIN_TEXTURE_CACHE_BYTES: u64 = 256 * MB;
const MAX_TEXTURE_CACHE_BYTES: u64 = 4096 * MB;
const TEXTURE_CACHE_SHARE: u64 = 8;

#[derive(Debug, Clone, Default)]
pub struct GpuRendererOptions {
    pub texture_cache_max_bytes: Option<u64>,
    pub memory_budget_bytes: Option<u64>,
    #[cfg(feature = "native")]
    pub host: HostMemory,
    #[cfg(feature = "native")]
    pub tiles: Option<TilePlan>,
    pub timestamps: bool,
}

impl GpuRendererOptions {
    #[cfg(feature = "native")]
    pub(super) fn host_budget(&self) -> Option<u64> {
        self.host.budget()
    }

    #[cfg(not(feature = "native"))]
    pub(super) fn host_budget(&self) -> Option<u64> {
        None
    }

    pub(super) fn memory_budget(&self, device: Option<DeviceMemory>) -> Option<u64> {
        if self.memory_budget_bytes.is_some() {
            return self.memory_budget_bytes;
        }
        let device = device?;
        let usable = device.bytes * u64::from(RESOURCE_CREATION_BUDGET_PERCENT) / 100;
        Some(match self.host_budget() {
            Some(host) if device.unified => usable.min(host),
            _ => usable,
        })
    }

    pub(super) fn texture_cache_bytes(&self, budget: Option<u64>) -> u64 {
        self.texture_cache_max_bytes.unwrap_or_else(|| {
            budget.map_or(DEFAULT_TEXTURE_CACHE_BYTES, |budget| {
                (budget / TEXTURE_CACHE_SHARE)
                    .clamp(MIN_TEXTURE_CACHE_BYTES, MAX_TEXTURE_CACHE_BYTES)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1024 * MB;

    fn device(bytes: u64, unified: bool) -> Option<DeviceMemory> {
        Some(DeviceMemory { bytes, unified })
    }

    fn host(budget: u64) -> GpuRendererOptions {
        GpuRendererOptions {
            host: HostMemory::new(Some(budget)),
            ..GpuRendererOptions::default()
        }
    }

    #[test]
    fn the_budget_is_the_override_else_the_scaled_probe_capped_on_unified_memory() {
        let cases = [
            (
                "override",
                GpuRendererOptions {
                    memory_budget_bytes: Some(GB),
                    ..host(GB / 2)
                },
                device(8 * GB, true),
                Some(GB),
            ),
            (
                "override without probe",
                GpuRendererOptions {
                    memory_budget_bytes: Some(GB),
                    ..Default::default()
                },
                None,
                Some(GB),
            ),
            (
                "discrete ignores the host",
                host(GB),
                device(10 * GB, false),
                Some(9 * GB),
            ),
            (
                "unified under the host",
                host(16 * GB),
                device(10 * GB, true),
                Some(9 * GB),
            ),
            (
                "unified over the host",
                host(4 * GB),
                device(10 * GB, true),
                Some(4 * GB),
            ),
            (
                "unified without a host",
                GpuRendererOptions::default(),
                device(10 * GB, true),
                Some(9 * GB),
            ),
            ("no probe", host(4 * GB), None, None),
        ];
        for (name, options, device, expected) in cases {
            let budget = options.memory_budget(device);
            if budget != expected {
                panic!("{name}: budget {budget:?}, expected {expected:?}");
            }
        }
    }

    #[test]
    fn the_texture_cache_defaults_to_an_eighth_of_the_budget_within_its_range() {
        let cases = [
            (None, None, 512 * MB),
            (Some(64 * MB), Some(32 * GB), 64 * MB),
            (None, Some(8 * GB), GB),
            (None, Some(GB), 256 * MB),
            (None, Some(64 * GB), 4 * GB),
        ];
        for (configured, budget, expected) in cases {
            let options = GpuRendererOptions {
                texture_cache_max_bytes: configured,
                ..Default::default()
            };
            let bytes = options.texture_cache_bytes(budget);
            if bytes != expected {
                panic!("{configured:?} with budget {budget:?} gave {bytes}, expected {expected}");
            }
        }
    }
}
