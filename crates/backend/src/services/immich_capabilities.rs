use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::immich::ImmichClient;
use crate::immich::client::{ServerConfig, ServerFeatures};
use crate::immich::dto::ServerVersion;

const TTL: Duration = Duration::from_secs(60);
const STRUCTURED_SEARCH: ServerVersion = ServerVersion {
    major: 3,
    minor: 2,
    patch: 0,
};
const TAG_RENAME: ServerVersion = ServerVersion {
    major: 3,
    minor: 3,
    patch: 0,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmichCapabilities {
    pub version: Option<ServerVersion>,
    pub structured_search: bool,
    pub tag_rename: bool,
    pub smart_search: bool,
    pub ocr: bool,
    pub trash: bool,
    pub trash_days: Option<u32>,
    pub external_host: Option<String>,
    pub map_light_style: Option<String>,
    pub map_dark_style: Option<String>,
}

fn style_url(raw: Option<&String>) -> Option<String> {
    raw.map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}

impl ImmichCapabilities {
    fn new(
        version: Option<ServerVersion>,
        features: Option<&ServerFeatures>,
        config: Option<&ServerConfig>,
    ) -> Self {
        Self {
            version,
            structured_search: version.is_some_and(|v| v >= STRUCTURED_SEARCH),
            tag_rename: version.is_some_and(|v| v >= TAG_RENAME),
            smart_search: features.is_none_or(|f| f.smart_search),
            ocr: features.is_none_or(|f| f.ocr),
            trash: features.is_some_and(|f| f.trash),
            trash_days: config.and_then(|c| c.trash_days),
            external_host: config
                .and_then(|c| c.external_domain.as_deref())
                .and_then(external_host),
            map_light_style: style_url(config.and_then(|c| c.map_light_style_url.as_ref())),
            map_dark_style: style_url(config.and_then(|c| c.map_dark_style_url.as_ref())),
        }
    }
}

fn external_host(domain: &str) -> Option<String> {
    let url = url::Url::parse(domain.trim()).ok()?;
    let host = url.host_str().filter(|host| !host.is_empty())?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    })
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
        let (version, features, config) = tokio::join!(
            immich.server_version(),
            immich.server_features(),
            immich.server_config()
        );
        let version = version
            .inspect_err(|e| tracing::warn!(error = %e, "immich server version unavailable"))
            .ok();
        let features = features
            .inspect_err(|e| tracing::warn!(error = %e, "immich server features unavailable"))
            .ok();
        let config = config
            .inspect_err(|e| tracing::warn!(error = %e, "immich server config unavailable"))
            .ok();
        let fresh = ImmichCapabilities::new(version, features.as_ref(), config.as_ref());
        if version.is_some()
            && features.is_some()
            && config.is_some()
            && let Ok(mut guard) = self.inner.lock()
        {
            guard.insert(server_epoch, (Instant::now(), fresh.clone()));
        }
        fresh
    }

    fn cached(&self, server_epoch: i64) -> Option<ImmichCapabilities> {
        let guard = self.inner.lock().ok()?;
        let (at, caps) = guard.get(&server_epoch)?;
        (at.elapsed() < TTL).then(|| caps.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::external_host;

    #[test]
    fn an_external_domain_names_its_host_and_port() {
        for (domain, host) in [
            ("https://photos.example.net", Some("photos.example.net")),
            ("https://photos.example.net/", Some("photos.example.net")),
            (
                "http://photos.example.net:2283/immich",
                Some("photos.example.net:2283"),
            ),
            ("https://photos.example.net:443", Some("photos.example.net")),
            (" https://photos.example.net ", Some("photos.example.net")),
            ("", None),
            ("   ", None),
            ("photos.example.net", None),
            ("not a url", None),
        ] {
            assert_eq!(external_host(domain).as_deref(), host, "{domain:?}");
        }
    }
}
