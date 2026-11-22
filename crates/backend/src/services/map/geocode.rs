use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use super::photon::{Place, parse};
use crate::services::owner_window::{Admission, OwnerWindow};
use crate::services::ttl_cache::TtlCache;
use crate::telemetry::ErrorChain;

const LIMIT: usize = 6;
const MAX_QUERY_CHARS: usize = 120;
const MIN_QUERY_CHARS: usize = 2;
const SEARCHES_PER_MINUTE: u32 = 60;
const CACHE_ENTRIES: NonZeroUsize = NonZeroUsize::new(512).unwrap();
const CACHE_TTL: Duration = Duration::from_secs(3600);
const TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Debug, thiserror::Error)]
pub enum GeocodeError {
    #[error("place search is off")]
    Off,
    #[error("search text must be {MIN_QUERY_CHARS} to {MAX_QUERY_CHARS} characters")]
    BadQuery,
    #[error("too many place searches")]
    Throttled,
    #[error("place search failed: {0}")]
    Upstream(String),
}

#[derive(Clone)]
pub struct Geocoder {
    http: reqwest::Client,
    cache: TtlCache<(String, String), Vec<Place>>,
    window: Arc<OwnerWindow>,
}

impl Default for Geocoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Geocoder {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .user_agent("immich-edit")
                .build()
                .unwrap_or_default(),
            cache: TtlCache::new(CACHE_ENTRIES, CACHE_TTL),
            window: Arc::new(OwnerWindow::new(
                SEARCHES_PER_MINUTE,
                Duration::from_secs(60),
            )),
        }
    }

    pub async fn search(
        &self,
        base: &str,
        owner: Uuid,
        raw: &str,
    ) -> Result<Vec<Place>, GeocodeError> {
        if base.is_empty() {
            return Err(GeocodeError::Off);
        }
        let query = raw.trim();
        let chars = query.chars().count();
        if !(MIN_QUERY_CHARS..=MAX_QUERY_CHARS).contains(&chars)
            || query.chars().any(char::is_control)
        {
            return Err(GeocodeError::BadQuery);
        }
        let key = (base.to_string(), query.to_lowercase());
        if let Some(hit) = self.cache.get(&key).await {
            return Ok(hit);
        }
        if self.window.admit(owner) != Admission::Accept {
            return Err(GeocodeError::Throttled);
        }
        let places = self.fetch(base, query).await?;
        self.cache.put(key, places.clone()).await;
        Ok(places)
    }

    async fn fetch(&self, base: &str, query: &str) -> Result<Vec<Place>, GeocodeError> {
        let url = format!("{base}/api");
        let limit = LIMIT.to_string();
        let resp = self
            .http
            .get(&url)
            .query(&[("q", query), ("limit", limit.as_str())])
            .send()
            .await
            .map_err(|e| upstream(&e.without_url()))?;
        let status = resp.status();
        if !status.is_success() {
            tracing::warn!(target: "upstream", status = status.as_u16(), "place search refused");
            return Err(GeocodeError::Upstream(format!("HTTP {}", status.as_u16())));
        }
        let body = resp.bytes().await.map_err(|e| upstream(&e.without_url()))?;
        let mut places = parse(&body).map_err(|e| upstream(&e))?;
        places.truncate(LIMIT);
        Ok(places)
    }
}

fn upstream(err: &(dyn std::error::Error + 'static)) -> GeocodeError {
    let detail = ErrorChain(err).to_string();
    tracing::warn!(target: "upstream", error = %detail, "place search failed");
    GeocodeError::Upstream(detail)
}
