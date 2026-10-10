use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::services::search::Searcher;
use crate::services::search::filter::{BrowseFilter, Scope};

const FETCH_CONCURRENCY: usize = 6;
const CACHE_CAP: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CountKey {
    owner: Uuid,
    server_epoch: i64,
    entity_id: Uuid,
}

#[derive(Clone)]
pub struct AssetCountCache {
    scope: fn(Uuid) -> Scope,
    inner: Arc<Mutex<LruCache<CountKey, u64>>>,
}

impl AssetCountCache {
    pub fn new(scope: fn(Uuid) -> Scope) -> Self {
        Self {
            scope,
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CAP).unwrap(),
            ))),
        }
    }

    pub async fn invalidate(&self, owner: Uuid, server_epoch: i64, id: Uuid) {
        self.inner.lock().await.pop(&CountKey {
            owner,
            server_epoch,
            entity_id: id,
        });
    }

    pub async fn clear_tenant(&self, owner: Uuid, server_epoch: i64) {
        let mut cache = self.inner.lock().await;
        let stale: Vec<CountKey> = cache
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| key.owner == owner && key.server_epoch == server_epoch)
            .collect();
        for key in stale {
            cache.pop(&key);
        }
    }

    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }

    pub async fn counts_for(
        &self,
        owner: Uuid,
        server_epoch: i64,
        searcher: &Searcher,
        ids: &[Uuid],
    ) -> HashMap<Uuid, u64> {
        let mut result: HashMap<Uuid, u64> = HashMap::new();
        let mut missing: Vec<Uuid> = Vec::new();
        {
            let mut cache = self.inner.lock().await;
            for id in ids {
                let key = CountKey {
                    owner,
                    server_epoch,
                    entity_id: *id,
                };
                match cache.get(&key) {
                    Some(count) => {
                        result.insert(*id, *count);
                    }
                    None => missing.push(*id),
                }
            }
        }

        let mut tasks: JoinSet<(Uuid, Option<u64>)> = JoinSet::new();
        let mut queue = missing.into_iter();
        for id in queue.by_ref().take(FETCH_CONCURRENCY) {
            spawn_count(&mut tasks, searcher.clone(), (self.scope)(id), id);
        }

        while let Some(joined) = tasks.join_next().await {
            let Ok((id, count)) = joined else {
                continue;
            };
            if let Some(count) = count {
                result.insert(id, count);
                self.inner.lock().await.put(
                    CountKey {
                        owner,
                        server_epoch,
                        entity_id: id,
                    },
                    count,
                );
            }
            if let Some(next) = queue.next() {
                spawn_count(&mut tasks, searcher.clone(), (self.scope)(next), next);
            }
        }

        result
    }
}

fn spawn_count(
    tasks: &mut JoinSet<(Uuid, Option<u64>)>,
    searcher: Searcher,
    scope: Scope,
    id: Uuid,
) {
    tasks.spawn(async move {
        let filter = BrowseFilter {
            scope: Some(scope),
            ..BrowseFilter::default()
        };
        match searcher.count(&filter).await {
            Ok(total) => (id, Some(total)),
            Err(e) => {
                tracing::warn!(?scope, error = %e, "asset count lookup failed");
                (id, None)
            }
        }
    });
}
