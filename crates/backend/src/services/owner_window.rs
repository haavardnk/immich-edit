use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;

const MAX_OWNERS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Accept,
    FirstDrop,
    Drop,
}

struct Window {
    start: Instant,
    count: u32,
}

pub struct OwnerWindow {
    limit: u32,
    window: Duration,
    inner: Mutex<HashMap<Uuid, Window>>,
}

impl OwnerWindow {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            inner: Mutex::default(),
        }
    }

    pub fn admit(&self, owner: Uuid) -> Admission {
        self.admit_at(owner, Instant::now())
    }

    fn admit_at(&self, owner: Uuid, now: Instant) -> Admission {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() >= MAX_OWNERS && !map.contains_key(&owner) {
            map.retain(|_, w| now.duration_since(w.start) < self.window);
        }
        let window = map.entry(owner).or_insert(Window {
            start: now,
            count: 0,
        });
        if now.duration_since(window.start) >= self.window {
            window.start = now;
            window.count = 0;
        }
        window.count = window.count.saturating_add(1);
        match window.count {
            count if count <= self.limit => Admission::Accept,
            count if count == self.limit + 1 => Admission::FirstDrop,
            _ => Admission::Drop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Admission, OwnerWindow};
    use std::time::{Duration, Instant};
    use uuid::Uuid;

    #[test]
    fn drops_after_limit_and_resets_per_window() {
        const LIMIT: u32 = 30;
        const WINDOW: Duration = Duration::from_secs(60);
        let limiter = OwnerWindow::new(LIMIT, WINDOW);
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let start = Instant::now();
        let admitted: Vec<Admission> = (0..LIMIT + 2)
            .map(|_| limiter.admit_at(owner, start))
            .collect();
        let accepted = admitted.iter().filter(|a| **a == Admission::Accept).count();
        if accepted != LIMIT as usize {
            panic!("expected {LIMIT} accepted, got {accepted}");
        }
        if admitted[LIMIT as usize..] != [Admission::FirstDrop, Admission::Drop] {
            panic!("over-limit reports must drop, warning once: {admitted:?}");
        }
        if limiter.admit_at(other, start) != Admission::Accept {
            panic!("limits are per owner");
        }
        let later = start + WINDOW + Duration::from_millis(1);
        if limiter.admit_at(owner, later) != Admission::Accept {
            panic!("a new window must accept again");
        }
    }
}
