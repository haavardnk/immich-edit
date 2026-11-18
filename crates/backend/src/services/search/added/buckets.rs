use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::services::search::Searcher;
use crate::services::search::added::sources::Params;
use crate::services::ttl_cache::TtlCache;

const CACHE_CAP: NonZeroUsize = NonZeroUsize::new(64).unwrap();
const CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone, PartialEq, Eq, Hash)]
struct BucketKey {
    owner: Uuid,
    server_epoch: i64,
    params: String,
    bucket: String,
}

#[derive(Clone)]
pub struct TimelineCache {
    inner: TtlCache<BucketKey, Arc<[Entry]>>,
}

impl Default for TimelineCache {
    fn default() -> Self {
        Self {
            inner: TtlCache::new(CACHE_CAP, CACHE_TTL),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Entry {
    pub at: DateTime<Utc>,
    pub id: Uuid,
}

pub(super) async fn entries(
    searcher: &Searcher,
    params: &Params,
    bucket: &str,
    count: u64,
) -> Result<Arc<[Entry]>, AppError> {
    let key = BucketKey {
        owner: searcher.owner,
        server_epoch: searcher.server_epoch,
        params: params
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("&"),
        bucket: bucket.to_owned(),
    };
    let cache = &searcher.caches.timelines.inner;
    if let Some(hit) = cache.get(&key).await
        && hit.len() as u64 == count
    {
        return Ok(hit);
    }
    let assets = searcher.immich.timeline_bucket(params, bucket).await?;
    if assets.id.len() != assets.created_at.len() {
        return Err(ImmichError::Decode("timeline bucket columns differ in length".into()).into());
    }
    let entries = assets
        .id
        .into_iter()
        .zip(assets.created_at)
        .map(|(id, raw)| {
            parse_at(&raw)
                .map(|at| Entry { at, id })
                .ok_or_else(|| ImmichError::Decode(format!("timeline createdAt {raw:?}")))
        })
        .collect::<Result<Arc<[Entry]>, ImmichError>>()?;
    cache.put(key, entries.clone()).await;
    Ok(entries)
}

pub fn parse_at(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|at| at.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|at| at.and_utc())
        })
}
