use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_ENTRIES: usize = 10_000;

#[derive(Clone, Copy)]
struct Bucket {
    max_failures: u32,
    window: Duration,
}

const PER_CLIENT: Bucket = Bucket {
    max_failures: 5,
    window: Duration::from_secs(15 * 60),
};

const PER_IDENTITY: Bucket = Bucket {
    max_failures: 25,
    window: Duration::from_secs(60 * 60),
};

pub struct LoginKey {
    per_client: String,
    per_identity: Option<String>,
}

impl LoginKey {
    pub fn identity(scope: &str, ip: &str, identity: &str) -> Self {
        let identity = identity.to_lowercase();
        Self {
            per_client: format!("{ip}|{scope}|{identity}"),
            per_identity: Some(format!("{scope}|{identity}")),
        }
    }

    pub fn client(scope: &str, ip: &str) -> Self {
        Self {
            per_client: format!("{ip}|{scope}"),
            per_identity: None,
        }
    }
}

struct Entry {
    failures: u32,
    window_start: Instant,
    window: Duration,
    locked_until: Option<Instant>,
}

pub struct LoginLimiter {
    inner: Mutex<HashMap<String, Entry>>,
    per_client: Bucket,
    per_identity: Bucket,
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
            per_client: PER_CLIENT,
            per_identity: PER_IDENTITY,
        }
    }

    pub fn retry_after(&self, key: &LoginKey) -> Option<Duration> {
        let map = self.inner.lock().unwrap();
        let now = Instant::now();
        let client = locked_for(&map, &key.per_client, now);
        let identity = key
            .per_identity
            .as_deref()
            .and_then(|k| locked_for(&map, k, now));
        match (client, identity) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    pub fn record_failure(&self, key: &LoginKey) {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();
        evict(&mut map, now);
        bump(&mut map, &key.per_client, self.per_client, now);
        if let Some(identity) = key.per_identity.as_deref() {
            bump(&mut map, identity, self.per_identity, now);
        }
    }

    pub fn record_success(&self, key: &LoginKey) {
        let mut map = self.inner.lock().unwrap();
        map.remove(&key.per_client);
        if let Some(identity) = key.per_identity.as_deref() {
            map.remove(identity);
        }
    }
}

fn locked_for(map: &HashMap<String, Entry>, key: &str, now: Instant) -> Option<Duration> {
    let until = map.get(key).and_then(|e| e.locked_until)?;
    if until > now {
        Some(until.duration_since(now))
    } else {
        None
    }
}

fn evict(map: &mut HashMap<String, Entry>, now: Instant) {
    map.retain(|_, entry| {
        now.duration_since(entry.window_start) <= entry.window
            || entry.locked_until.is_some_and(|until| until > now)
    });
    if map.len() >= MAX_ENTRIES
        && let Some(oldest) = map
            .iter()
            .min_by_key(|(_, entry)| entry.window_start)
            .map(|(key, _)| key.clone())
    {
        map.remove(&oldest);
    }
}

fn bump(map: &mut HashMap<String, Entry>, key: &str, bucket: Bucket, now: Instant) {
    let entry = map.entry(key.to_string()).or_insert(Entry {
        failures: 0,
        window_start: now,
        window: bucket.window,
        locked_until: None,
    });
    if now.duration_since(entry.window_start) > bucket.window {
        entry.failures = 0;
        entry.window_start = now;
        entry.locked_until = None;
    }
    entry.failures += 1;
    if entry.failures >= bucket.max_failures {
        entry.locked_until = Some(now + bucket.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(ip: &str, identity: &str) -> LoginKey {
        LoginKey::identity("login", ip, identity)
    }

    #[test]
    fn unknown_key_is_not_locked() {
        let limiter = LoginLimiter::new();
        if limiter.retry_after(&key("1.2.3.4", "a@b")).is_some() {
            panic!("a fresh key must not be locked");
        }
    }

    #[test]
    fn locks_one_client_after_five_failures() {
        let limiter = LoginLimiter::new();
        let k = key("1.2.3.4", "a@b");
        for _ in 0..PER_CLIENT.max_failures - 1 {
            limiter.record_failure(&k);
            if limiter.retry_after(&k).is_some() {
                panic!("locked too early");
            }
        }
        limiter.record_failure(&k);
        if limiter.retry_after(&k).is_none() {
            panic!("client must lock on the fifth failure");
        }
        if limiter.retry_after(&key("5.6.7.8", "a@b")).is_some() {
            panic!("another client must still be able to try");
        }
    }

    #[test]
    fn locks_an_identity_across_rotating_clients() {
        let limiter = LoginLimiter::new();
        for n in 0..PER_IDENTITY.max_failures {
            limiter.record_failure(&key(&format!("10.0.0.{n}"), "victim@b"));
        }
        if limiter
            .retry_after(&key("203.0.113.9", "victim@b"))
            .is_none()
        {
            panic!("identity must lock once the looser bucket fills");
        }
        if limiter
            .retry_after(&key("203.0.113.9", "other@b"))
            .is_some()
        {
            panic!("an unrelated identity must be unaffected");
        }
    }

    #[test]
    fn a_client_without_an_identity_has_no_shared_bucket() {
        let limiter = LoginLimiter::new();
        for n in 0..PER_IDENTITY.max_failures {
            limiter.record_failure(&LoginKey::client("apikey", &format!("10.0.0.{n}")));
        }
        if limiter
            .retry_after(&LoginKey::client("apikey", "203.0.113.9"))
            .is_some()
        {
            panic!("api-key logins must not share a global bucket");
        }
    }

    #[test]
    fn success_clears_both_buckets() {
        let limiter = LoginLimiter::new();
        let k = key("1.2.3.4", "a@b");
        for _ in 0..PER_CLIENT.max_failures {
            limiter.record_failure(&k);
        }
        limiter.record_success(&k);
        if limiter.retry_after(&k).is_some() {
            panic!("success must clear the lock");
        }
    }

    #[test]
    fn lockout_expires_with_the_window() {
        let limiter = LoginLimiter {
            inner: Mutex::new(HashMap::new()),
            per_client: Bucket {
                max_failures: 2,
                window: Duration::from_millis(60),
            },
            per_identity: PER_IDENTITY,
        };
        let k = key("1.2.3.4", "a@b");
        limiter.record_failure(&k);
        limiter.record_failure(&k);
        if limiter.retry_after(&k).is_none() {
            panic!("must lock at the bucket maximum");
        }
        std::thread::sleep(Duration::from_millis(80));
        if limiter.retry_after(&k).is_some() {
            panic!("lock must expire once the window passes");
        }
    }
}
