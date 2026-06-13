use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::immich::ImmichClient;

const FETCH_CONCURRENCY: usize = 6;

#[derive(Clone)]
pub struct TagCountCache {
    inner: Arc<Mutex<HashMap<Uuid, u64>>>,
}

impl Default for TagCountCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TagCountCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn invalidate(&self, tag_id: Uuid) {
        self.inner.lock().await.remove(&tag_id);
    }

    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }

    pub async fn counts_for(&self, immich: &ImmichClient, tag_ids: &[Uuid]) -> HashMap<Uuid, u64> {
        let mut result: HashMap<Uuid, u64> = HashMap::new();
        let mut missing: Vec<Uuid> = Vec::new();
        {
            let cache = self.inner.lock().await;
            for id in tag_ids {
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
            spawn_count(&mut tasks, immich.clone(), id);
        }

        while let Some(joined) = tasks.join_next().await {
            let Ok((tag_id, count)) = joined else {
                continue;
            };
            if let Some(count) = count {
                result.insert(tag_id, count);
                self.inner.lock().await.insert(tag_id, count);
            }
            if let Some(next) = queue.next() {
                spawn_count(&mut tasks, immich.clone(), next);
            }
        }

        result
    }
}

fn spawn_count(tasks: &mut JoinSet<(Uuid, Option<u64>)>, immich: ImmichClient, tag_id: Uuid) {
    tasks.spawn(async move {
        let body = serde_json::json!({ "tagIds": [tag_id] });
        match immich.search_statistics(&body).await {
            Ok(stats) => (tag_id, Some(stats.total)),
            Err(e) => {
                tracing::warn!(%tag_id, error = %e, "tag asset count lookup failed");
                (tag_id, None)
            }
        }
    });
}
