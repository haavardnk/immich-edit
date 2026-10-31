use std::hash::Hash;
use std::sync::Arc;

use lru::LruCache;

pub(crate) fn unshared<T>(entry: &Arc<T>) -> bool {
    Arc::strong_count(entry) == 1
}

pub(crate) fn idle_bytes<K: Hash + Eq, V>(
    cache: &LruCache<K, V>,
    idle: impl Fn(&V) -> bool,
    bytes: impl Fn(&V) -> u64,
) -> u64 {
    cache
        .iter()
        .filter(|(_, entry)| idle(entry))
        .map(|(_, entry)| bytes(entry))
        .sum()
}

pub(crate) fn evict_idle<K: Hash + Eq + Clone, V>(
    cache: &mut LruCache<K, V>,
    idle: impl Fn(&V) -> bool,
    bytes: impl Fn(&V) -> u64,
    excess: u64,
) -> u64 {
    let keys: Vec<K> = cache
        .iter()
        .rev()
        .filter(|(_, entry)| idle(entry))
        .map(|(key, _)| key.clone())
        .collect();
    let mut freed = 0;
    for key in keys {
        if freed >= excess {
            break;
        }
        freed += cache.pop(&key).map_or(0, |entry| bytes(&entry));
    }
    freed
}
