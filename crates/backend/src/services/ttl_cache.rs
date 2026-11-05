use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use tokio::sync::Mutex;

pub struct TtlCache<K, V> {
    ttl: Duration,
    inner: Arc<Mutex<LruCache<K, (Instant, V)>>>,
}

impl<K, V> Clone for TtlCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            ttl: self.ttl,
            inner: self.inner.clone(),
        }
    }
}

impl<K: Hash + Eq, V: Clone> TtlCache<K, V> {
    pub fn new(capacity: NonZeroUsize, ttl: Duration) -> Self {
        Self {
            ttl,
            inner: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.lock().await;
        let (stored, value) = cache.get(key)?;
        if stored.elapsed() < self.ttl {
            return Some(value.clone());
        }
        cache.pop(key);
        None
    }

    pub async fn put(&self, key: K, value: V) {
        self.inner.lock().await.put(key, (Instant::now(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: NonZeroUsize = NonZeroUsize::MIN;
    const TWO: NonZeroUsize = NonZeroUsize::new(2).unwrap();

    #[tokio::test]
    async fn a_fresh_entry_is_served_until_it_expires() {
        let fresh = TtlCache::new(TWO, Duration::from_secs(60));
        fresh.put("a", 1).await;
        let expired = TtlCache::new(TWO, Duration::ZERO);
        expired.put("a", 1).await;
        let read = (fresh.get(&"a").await, expired.get(&"a").await);
        if read != (Some(1), None) {
            panic!("fresh and expired reads were {read:?}");
        }
    }

    #[tokio::test]
    async fn the_least_recently_used_entry_is_evicted() {
        let cache = TtlCache::new(ONE, Duration::from_secs(60));
        cache.put("a", 1).await;
        cache.put("b", 2).await;
        let read = (cache.get(&"a").await, cache.get(&"b").await);
        if read != (None, Some(2)) {
            panic!("reads after eviction were {read:?}");
        }
    }

    #[tokio::test]
    async fn clones_share_one_store() {
        let cache = TtlCache::new(TWO, Duration::from_secs(60));
        cache.clone().put("a", 1).await;
        if cache.get(&"a").await != Some(1) {
            panic!("a clone wrote to a separate store");
        }
    }
}
