use std::sync::OnceLock;

use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Serialize)]
pub struct HostInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub os_version: Option<String>,
    pub cpu: Option<String>,
    pub cores: usize,
    pub memory_total_bytes: Option<u64>,
    pub memory_limit_bytes: Option<u64>,
}

static HOST: OnceLock<HostInfo> = OnceLock::new();

pub fn host_info() -> &'static HostInfo {
    HOST.get_or_init(probe)
}

fn probe() -> HostInfo {
    let sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing())
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    );
    let total_memory = sys.total_memory();
    HostInfo {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        os_version: System::long_os_version(),
        cpu: sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|b| !b.is_empty()),
        // Respects the cgroup CPU quota, so this is what the container can actually use.
        cores: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0),
        memory_total_bytes: (total_memory > 0).then_some(total_memory),
        memory_limit_bytes: sys.cgroup_limits().map(|l| l.total_memory),
    }
}
