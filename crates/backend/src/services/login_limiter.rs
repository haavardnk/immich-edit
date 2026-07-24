use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_FAILURES: u32 = 5;
const WINDOW: Duration = Duration::from_secs(15 * 60);

struct Entry {
    failures: u32,
    window_start: Instant,
    locked_until: Option<Instant>,
}

pub struct LoginLimiter {
    inner: Mutex<HashMap<String, Entry>>,
}

impl Default for LoginLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl LoginLimiter {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn retry_after(&self, key: &str) -> Option<Duration> {
        let map = self.inner.lock().unwrap();
        let now = Instant::now();
        map.get(key).and_then(|e| e.locked_until).and_then(|until| {
            if until > now {
                Some(until.duration_since(now))
            } else {
                None
            }
        })
    }

    pub fn record_failure(&self, key: &str) {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_insert(Entry {
            failures: 0,
            window_start: now,
            locked_until: None,
        });
        if now.duration_since(entry.window_start) > WINDOW {
            entry.failures = 0;
            entry.window_start = now;
            entry.locked_until = None;
        }
        entry.failures += 1;
        if entry.failures >= MAX_FAILURES {
            entry.locked_until = Some(now + WINDOW);
        }
    }

    pub fn record_success(&self, key: &str) {
        self.inner.lock().unwrap().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_key_is_not_locked() {
        let limiter = LoginLimiter::new();
        assert!(limiter.retry_after("1.2.3.4|a@b").is_none());
    }

    #[test]
    fn locks_after_max_failures() {
        let limiter = LoginLimiter::new();
        for _ in 0..MAX_FAILURES - 1 {
            limiter.record_failure("k");
            assert!(limiter.retry_after("k").is_none());
        }
        limiter.record_failure("k");
        assert!(limiter.retry_after("k").is_some());
    }

    #[test]
    fn success_clears_failures() {
        let limiter = LoginLimiter::new();
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("k");
        }
        limiter.record_success("k");
        assert!(limiter.retry_after("k").is_none());
    }
}
