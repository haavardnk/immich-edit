use std::sync::Arc;

use ml::RuntimeMode;
use tokio::runtime::Handle;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::MlRuntimeMode;

pub fn runtime_mode(mode: MlRuntimeMode) -> RuntimeMode {
    match mode {
        MlRuntimeMode::Gpu => RuntimeMode::Gpu,
        MlRuntimeMode::Cpu => RuntimeMode::Cpu,
        _ => RuntimeMode::Auto,
    }
}

#[derive(Clone)]
pub struct InferencePermits(Arc<Semaphore>);

impl InferencePermits {
    pub fn new(max_concurrency: usize) -> Self {
        Self(Arc::new(Semaphore::new(max_concurrency.max(1))))
    }

    pub async fn acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.0.clone().acquire_owned().await.ok()
    }

    pub fn blocking_acquire(&self) -> Option<OwnedSemaphorePermit> {
        let handle = Handle::try_current().ok()?;
        handle.block_on(self.0.clone().acquire_owned()).ok()
    }
}
