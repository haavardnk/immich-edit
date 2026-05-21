use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

pub trait ByteSized {
    fn byte_size(&self) -> usize;
}

pub struct ByteLru<K, V> {
    capacity: usize,
    used: usize,
    order: Vec<K>,
    map: HashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: ByteSized + Clone> ByteLru<K, V> {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            capacity: capacity_bytes,
            used: 0,
            order: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let value = self.map.get(key)?.clone();
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let k = self.order.remove(pos);
            self.order.push(k);
        }
        Some(value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let size = value.byte_size();
        if let Some(old) = self.map.remove(&key) {
            self.used = self.used.saturating_sub(old.byte_size());
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
        }
        while self.used + size > self.capacity && !self.order.is_empty() {
            let evict = self.order.remove(0);
            if let Some(v) = self.map.remove(&evict) {
                self.used = self.used.saturating_sub(v.byte_size());
            }
        }
        self.used += size;
        self.order.push(key.clone());
        self.map.insert(key, value);
    }

    pub fn used_bytes(&self) -> usize {
        self.used
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub struct SharedByteLru<K, V> {
    inner: Mutex<ByteLru<K, V>>,
}

impl<K: Eq + Hash + Clone, V: ByteSized + Clone> SharedByteLru<K, V> {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            inner: Mutex::new(ByteLru::new(capacity_bytes)),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.lock().ok()?.get(key)
    }

    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Blob(Vec<u8>);

    impl ByteSized for Blob {
        fn byte_size(&self) -> usize {
            self.0.len()
        }
    }

    #[test]
    fn evicts_when_over_capacity() {
        let mut lru: ByteLru<String, Blob> = ByteLru::new(100);
        lru.insert("a".into(), Blob(vec![0; 40]));
        lru.insert("b".into(), Blob(vec![0; 40]));
        lru.insert("c".into(), Blob(vec![0; 40]));
        if lru.used_bytes() > 100 {
            panic!("over capacity: {}", lru.used_bytes());
        }
        if lru.get(&"a".to_string()).is_some() {
            panic!("a should be evicted");
        }
    }

    #[test]
    fn get_promotes_recency() {
        let mut lru: ByteLru<String, Blob> = ByteLru::new(100);
        lru.insert("a".into(), Blob(vec![0; 40]));
        lru.insert("b".into(), Blob(vec![0; 40]));
        lru.get(&"a".to_string());
        lru.insert("c".into(), Blob(vec![0; 40]));
        if lru.get(&"a".to_string()).is_none() {
            panic!("a should survive (promoted)");
        }
        if lru.get(&"b".to_string()).is_some() {
            panic!("b should be evicted");
        }
    }
}
