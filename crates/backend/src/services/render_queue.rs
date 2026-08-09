use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use lru::LruCache;
use raw_pipeline::CancelTracker;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

use crate::asset_key::AssetKey;

const TRACKER_CAP: usize = 1024;
const LATEST_CAP: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderKey {
    pub owner: Uuid,
    pub server_epoch: i64,
    pub asset_id: AssetKey,
}

#[derive(Clone)]
pub struct RenderQueue {
    max_concurrency: usize,
    semaphore: Arc<Semaphore>,
    latest: Arc<Mutex<LruCache<RenderKey, u64>>>,
    trackers: Arc<Mutex<LruCache<RenderKey, CancelTracker>>>,
}

impl RenderQueue {
    pub fn new(max_concurrency: usize) -> Self {
        let cap = max_concurrency.max(1);
        Self {
            max_concurrency: cap,
            semaphore: Arc::new(Semaphore::new(cap)),
            latest: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(LATEST_CAP).unwrap(),
            ))),
            trackers: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(TRACKER_CAP).unwrap(),
            ))),
        }
    }

    pub async fn tracker(&self, key: RenderKey) -> CancelTracker {
        let mut map = self.trackers.lock().await;
        if let Some(t) = map.get(&key) {
            return t.clone();
        }
        let t = CancelTracker::default();
        map.put(key, t.clone());
        t
    }

    pub async fn enqueue<F, T, E>(&self, key: RenderKey, work: F) -> Option<Result<T, E>>
    where
        F: std::future::Future<Output = Result<T, E>> + Send,
    {
        let ticket: u64 = {
            let mut map = self.latest.lock().await;
            let counter = map.get_or_insert_mut(key, || 0u64);
            *counter = counter.wrapping_add(1);
            *counter
        };

        let permit = self.semaphore.clone().acquire_owned().await.ok()?;
        {
            let mut map = self.latest.lock().await;
            if map.get(&key).copied() != Some(ticket) {
                drop(permit);
                return None;
            }
        }

        let result = work.await;
        drop(permit);
        Some(result)
    }

    pub async fn shutdown(&self, timeout: Duration) {
        self.semaphore.close();
        {
            let trackers = self.trackers.lock().await;
            for (_, t) in trackers.iter() {
                t.cancel_all();
            }
        }
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.semaphore.available_permits() >= self.max_concurrency {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn cancel_all(&self) {
        let trackers = self.trackers.lock().await;
        for (_, tracker) in trackers.iter() {
            tracker.cancel_all();
        }
    }

    pub async fn cancel_owner(&self, owner: Uuid) {
        let trackers = self.trackers.lock().await;
        for (key, tracker) in trackers.iter() {
            if key.owner == owner {
                tracker.cancel_all();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn key(asset_id: Uuid) -> RenderKey {
        RenderKey {
            owner: Uuid::nil(),
            server_epoch: 1,
            asset_id: AssetKey::master(asset_id),
        }
    }

    #[tokio::test]
    async fn latest_wins_collapses_pending() {
        let q = RenderQueue::new(1);
        let id = key(Uuid::new_v4());
        let runs = Arc::new(AtomicUsize::new(0));

        let q1 = q.clone();
        let r1 = runs.clone();
        let h1 = tokio::spawn(async move {
            q1.enqueue::<_, &'static str, ()>(id, async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                r1.fetch_add(1, Ordering::SeqCst);
                Ok("first")
            })
            .await
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let q2 = q.clone();
        let r2 = runs.clone();
        let h2 = tokio::spawn(async move {
            q2.enqueue::<_, &'static str, ()>(id, async move {
                r2.fetch_add(1, Ordering::SeqCst);
                Ok("second")
            })
            .await
        });
        let q3 = q.clone();
        let r3 = runs.clone();
        let h3 = tokio::spawn(async move {
            q3.enqueue::<_, &'static str, ()>(id, async move {
                r3.fetch_add(1, Ordering::SeqCst);
                Ok("third")
            })
            .await
        });

        let (a, b, c) = tokio::join!(h1, h2, h3);
        let a = a.unwrap();
        let b = b.unwrap();
        let c = c.unwrap();

        let mut completions = 0;
        for r in [a, b, c] {
            if let Some(Ok(_)) = r {
                completions += 1;
            }
        }
        if !(1..=2).contains(&completions) {
            panic!("expected 1..=2 completions, got {completions}");
        }
        if runs.load(Ordering::SeqCst) > 2 {
            panic!("collapsed too few: ran {}", runs.load(Ordering::SeqCst));
        }
    }

    #[tokio::test]
    async fn shutdown_cancels_trackers_and_drains() {
        let q = RenderQueue::new(2);
        let a = key(Uuid::new_v4());
        let b = key(Uuid::new_v4());
        let token_a = q.tracker(a).await.next();
        let token_b = q.tracker(b).await.next();

        let q1 = q.clone();
        let h = tokio::spawn(async move {
            q1.enqueue::<_, &'static str, ()>(a, async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok("done")
            })
            .await
        });

        tokio::time::sleep(Duration::from_millis(5)).await;
        q.shutdown(Duration::from_secs(2)).await;

        if !token_a.is_cancelled() || !token_b.is_cancelled() {
            panic!("trackers not cancelled");
        }
        let _ = h.await.unwrap();
        if q.semaphore.available_permits() < 2 {
            panic!("did not drain");
        }
        let after = q
            .enqueue::<_, &'static str, ()>(key(Uuid::new_v4()), async move { Ok("late") })
            .await;
        if after.is_some() {
            panic!("post-shutdown enqueue should be rejected");
        }
    }
}
