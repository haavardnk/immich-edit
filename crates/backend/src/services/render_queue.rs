use std::collections::HashMap;
use std::sync::Arc;

use raw_pipeline::CancelTracker;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

#[derive(Clone)]
pub struct RenderQueue {
    semaphore: Arc<Semaphore>,
    latest: Arc<Mutex<HashMap<Uuid, u64>>>,
    trackers: Arc<Mutex<HashMap<Uuid, CancelTracker>>>,
}

impl RenderQueue {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency.max(1))),
            latest: Arc::new(Mutex::new(HashMap::new())),
            trackers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn tracker(&self, asset_id: Uuid) -> CancelTracker {
        let mut map = self.trackers.lock().await;
        map.entry(asset_id).or_default().clone()
    }

    pub async fn enqueue<F, T, E>(&self, asset_id: Uuid, work: F) -> Option<Result<T, E>>
    where
        F: std::future::Future<Output = Result<T, E>> + Send,
    {
        let ticket: u64 = {
            let mut map = self.latest.lock().await;
            let counter = map.entry(asset_id).or_insert(0);
            *counter = counter.wrapping_add(1);
            *counter
        };

        let permit = self.semaphore.clone().acquire_owned().await.ok()?;
        {
            let map = self.latest.lock().await;
            if map.get(&asset_id).copied() != Some(ticket) {
                drop(permit);
                return None;
            }
        }

        let result = work.await;
        drop(permit);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn latest_wins_collapses_pending() {
        let q = RenderQueue::new(1);
        let id = Uuid::new_v4();
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
}
