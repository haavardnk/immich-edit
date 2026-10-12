use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::immich::ImmichClient;
use crate::immich::client::ServerFeatures;
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
    pub smart_search: bool,
    pub ocr: bool,
}

impl ImmichCapabilities {
    fn new(version: Option<ServerVersion>, features: Option<&ServerFeatures>) -> Self {
        Self {
            version,
            structured_search: version.is_some_and(|v| v >= STRUCTURED_SEARCH),
            smart_search: features.is_none_or(|f| f.smart_search),
            ocr: features.is_none_or(|f| f.ocr),
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
        let (version, features) = tokio::join!(immich.server_version(), immich.server_features());
        let version = version
            .inspect_err(|e| tracing::warn!(error = %e, "immich server version unavailable"))
            .ok();
        let features = features
            .inspect_err(|e| tracing::warn!(error = %e, "immich server features unavailable"))
            .ok();
        let fresh = ImmichCapabilities::new(version, features.as_ref());
        if version.is_some()
            && features.is_some()
            && let Ok(mut guard) = self.inner.lock()
        {
            guard.insert(server_epoch, (Instant::now(), fresh));
        }
        fresh
    }

    fn cached(&self, server_epoch: i64) -> Option<ImmichCapabilities> {
        let guard = self.inner.lock().ok()?;
        let (at, caps) = guard.get(&server_epoch)?;
        (at.elapsed() < TTL).then_some(*caps)
    }
}
