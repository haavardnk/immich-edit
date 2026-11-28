use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::{StreamExt, TryStreamExt, stream};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{AlbumSummary, TagSummary};
use crate::immich::{ImmichClient, ImmichError, ImmichResult};
use crate::services::ttl_cache::TtlCache;

const CACHE_CAP: NonZeroUsize = NonZeroUsize::new(4096).unwrap();
const CACHE_TTL: Duration = Duration::from_secs(60);
const CONCURRENCY: usize = 8;

pub trait Source {
    type Value: Clone + Send + Sync + 'static;

    fn fetch(
        immich: &ImmichClient,
        asset: Uuid,
    ) -> impl Future<Output = ImmichResult<Self::Value>> + Send;
}

pub struct Albums;

impl Source for Albums {
    type Value = Vec<AlbumSummary>;

    async fn fetch(immich: &ImmichClient, asset: Uuid) -> ImmichResult<Self::Value> {
        immich.albums_with_asset(asset).await
    }
}

pub struct Tags;

impl Source for Tags {
    type Value = Vec<TagSummary>;

    async fn fetch(immich: &ImmichClient, asset: Uuid) -> ImmichResult<Self::Value> {
        match immich.asset(asset).await {
            Ok(detail) => Ok(detail.tags),
            Err(ImmichError::NotFound | ImmichError::Rejected { status: 400, .. }) => {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Key {
    generation: u64,
    server_epoch: i64,
    owner: Uuid,
    asset: Uuid,
}

pub struct MembershipCache<S: Source> {
    stored: TtlCache<Key, S::Value>,
    generation: Arc<AtomicU64>,
}

impl<S: Source> Clone for MembershipCache<S> {
    fn clone(&self) -> Self {
        Self {
            stored: self.stored.clone(),
            generation: self.generation.clone(),
        }
    }
}

impl<S: Source> Default for MembershipCache<S> {
    fn default() -> Self {
        Self {
            stored: TtlCache::new(CACHE_CAP, CACHE_TTL),
            generation: Arc::default(),
        }
    }
}

pub struct Reader<'a> {
    pub immich: &'a ImmichClient,
    pub server_epoch: i64,
    pub owner: Uuid,
}

impl<S: Source> MembershipCache<S> {
    pub fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    pub async fn of(
        &self,
        reader: Reader<'_>,
        assets: Vec<Uuid>,
    ) -> Result<BTreeMap<Uuid, S::Value>, AppError> {
        let generation = self.generation.load(Ordering::SeqCst);
        let reader = &reader;
        stream::iter(assets)
            .map(|asset| {
                let key = Key {
                    generation,
                    server_epoch: reader.server_epoch,
                    owner: reader.owner,
                    asset,
                };
                async move { Ok((asset, self.lookup(reader.immich, key).await?)) }
            })
            .buffer_unordered(CONCURRENCY)
            .try_collect()
            .await
    }

    async fn lookup(&self, immich: &ImmichClient, key: Key) -> Result<S::Value, AppError> {
        if let Some(hit) = self.stored.get(&key).await {
            return Ok(hit);
        }
        let value = S::fetch(immich, key.asset).await?;
        self.stored.put(key, value.clone()).await;
        Ok(value)
    }
}
