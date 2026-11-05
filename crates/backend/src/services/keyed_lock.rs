use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

type Slots<K> = Arc<Mutex<HashMap<K, Arc<AsyncMutex<()>>>>>;

pub struct KeyedLocks<K> {
    slots: Slots<K>,
}

impl<K> Clone for KeyedLocks<K> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
        }
    }
}

impl<K> Default for KeyedLocks<K> {
    fn default() -> Self {
        Self {
            slots: Arc::default(),
        }
    }
}

pub struct KeyedGuard<K: Eq + Hash> {
    slots: Slots<K>,
    key: K,
    slot: Option<Arc<AsyncMutex<()>>>,
    held: Option<OwnedMutexGuard<()>>,
}

fn table<K>(slots: &Slots<K>) -> MutexGuard<'_, HashMap<K, Arc<AsyncMutex<()>>>> {
    slots.lock().unwrap_or_else(|e| e.into_inner())
}

impl<K: Eq + Hash + Clone> KeyedLocks<K> {
    pub async fn lock(&self, key: K) -> KeyedGuard<K> {
        let slot = table(&self.slots).entry(key.clone()).or_default().clone();
        let held = slot.clone().lock_owned().await;
        KeyedGuard {
            slots: self.slots.clone(),
            key,
            slot: Some(slot),
            held: Some(held),
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        table(&self.slots).len()
    }
}

impl<K: Eq + Hash> Drop for KeyedGuard<K> {
    fn drop(&mut self) {
        self.held.take();
        let mut slots = table(&self.slots);
        if let Some(slot) = self.slot.take()
            && Arc::strong_count(&slot) == 2
        {
            slots.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn one_key_runs_one_holder_at_a_time() {
        let locks = KeyedLocks::default();
        let first = locks.lock(1).await;

        let waiting = tokio::time::timeout(Duration::from_millis(50), locks.lock(1)).await;

        assert!(waiting.is_err());
        drop(first);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), locks.lock(1))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn different_keys_do_not_wait_for_each_other() {
        let locks = KeyedLocks::default();
        let _first = locks.lock(1).await;

        let other = tokio::time::timeout(Duration::from_millis(50), locks.lock(2)).await;

        assert!(other.is_ok());
    }

    #[tokio::test]
    async fn a_released_key_leaves_no_entry() {
        let locks = KeyedLocks::default();
        let first = locks.lock(1).await;
        let queued = tokio::spawn({
            let locks = locks.clone();
            async move {
                drop(locks.lock(1).await);
            }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;

        drop(first);
        queued.await.unwrap();

        assert_eq!(locks.len(), 0);
    }
}
