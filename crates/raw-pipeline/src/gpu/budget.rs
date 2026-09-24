use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct GpuBudget {
    max_bytes: u64,
    used: AtomicU64,
}

impl GpuBudget {
    pub fn new(max_bytes: u64) -> Arc<Self> {
        Arc::new(Self {
            max_bytes,
            used: AtomicU64::new(0),
        })
    }

    #[cfg(feature = "native")]
    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub fn try_reserve(&self, bytes: u64) -> bool {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                let next = used.saturating_add(bytes);
                (next <= self.max_bytes).then_some(next)
            })
            .is_ok()
    }

    pub fn release(&self, bytes: u64) {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                Some(used.saturating_sub(bytes))
            })
            .ok();
    }
}
