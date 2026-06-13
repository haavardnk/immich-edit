use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::immich::ImmichClient;

const FETCH_CONCURRENCY: usize = 6;

#[derive(Clone)]
pub struct AssetCountCache {
    field: &'static str,
    inner: Arc<Mutex<HashMap<Uuid, u64>>>,
}

impl AssetCountCache {
    pub fn new(field: &'static str) -> Self {
        Self {
            field,
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn invalidate(&self, id: Uuid) {
        self.inner.lock().await.remove(&id);
    }

    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }

    pub async fn counts_for(&self, immich: &ImmichClient, ids: &[Uuid]) -> HashMap<Uuid, u64> {
        let mut result: HashMap<Uuid, u64> = HashMap::new();
        let mut missing: Vec<Uuid> = Vec::new();
        {
            let cache = self.inner.lock().await;
            for id in ids {
                match cache.get(id) {
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
            spawn_count(&mut tasks, immich.clone(), self.field, id);
        }

        while let Some(joined) = tasks.join_next().await {
            let Ok((id, count)) = joined else {
                continue;
            };
            if let Some(count) = count {
                result.insert(id, count);
                self.inner.lock().await.insert(id, count);
            }
            if let Some(next) = queue.next() {
                spawn_count(&mut tasks, immich.clone(), self.field, next);
            }
        }

        result
    }
}

fn spawn_count(
    tasks: &mut JoinSet<(Uuid, Option<u64>)>,
    immich: ImmichClient,
    field: &'static str,
    id: Uuid,
) {
    tasks.spawn(async move {
        let body = serde_json::json!({ field: [id] });
        match immich.search_statistics(&body).await {
            Ok(stats) => (id, Some(stats.total)),
            Err(e) => {
                tracing::warn!(%id, field, error = %e, "asset count lookup failed");
                (id, None)
            }
        }
    });
}
