use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::immich::ImmichClient;
use crate::immich::dto::ServerVersion;

const TTL: Duration = Duration::from_secs(60);
const STRUCTURED_SEARCH: ServerVersion = ServerVersion {
    major: 3,
    minor: 2,
    patch: 0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImmichCapabilities {
    pub version: Option<ServerVersion>,
    pub structured_search: bool,
}

impl ImmichCapabilities {
    fn from_version(version: Option<ServerVersion>) -> Self {
        Self {
            version,
            structured_search: version.is_some_and(|v| v >= STRUCTURED_SEARCH),
        }
    }
}

#[derive(Clone, Default)]
pub struct CapabilityCache {
    inner: Arc<Mutex<HashMap<i64, (Instant, ImmichCapabilities)>>>,
}

impl CapabilityCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, server_epoch: i64, immich: &ImmichClient) -> ImmichCapabilities {
        if let Some(hit) = self.cached(server_epoch) {
            return hit;
        }
        match immich.server_version().await {
            Ok(version) => {
                let fresh = ImmichCapabilities::from_version(Some(version));
                if let Ok(mut guard) = self.inner.lock() {
                    guard.insert(server_epoch, (Instant::now(), fresh));
                }
                fresh
            }
            Err(e) => {
                tracing::warn!(error = %e, "immich server version unavailable");
                ImmichCapabilities::from_version(None)
            }
        }
    }

    fn cached(&self, server_epoch: i64) -> Option<ImmichCapabilities> {
        let guard = self.inner.lock().ok()?;
        let (at, caps) = guard.get(&server_epoch)?;
        (at.elapsed() < TTL).then_some(*caps)
    }
}
