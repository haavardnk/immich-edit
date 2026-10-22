use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tokio::sync::watch;
use uuid::Uuid;

use super::{DenoiseError, DenoiseStatus};
use crate::services::denoise_store::PlaneKey;

pub(super) const MAX_PENDING_PER_OWNER: usize = 8;
pub(super) const SETTLED_TTL: Duration = Duration::from_secs(600);

struct Entry {
    id: u64,
    rx: watch::Receiver<DenoiseStatus>,
    settled_at: Option<Instant>,
}

#[derive(Default)]
struct Inner {
    next_id: u64,
    entries: HashMap<PlaneKey, Entry>,
}

impl Inner {
    fn expire(&mut self, now: Instant) {
        self.entries.retain(|_, entry| {
            entry
                .settled_at
                .is_none_or(|at| now.saturating_duration_since(at) < SETTLED_TTL)
        });
    }

    fn pending_for(&self, owner: Uuid) -> usize {
        self.entries
            .iter()
            .filter(|(key, entry)| key.owner == owner && !entry.rx.borrow().settled())
            .count()
    }
}

pub(super) enum Admission {
    Existing(watch::Receiver<DenoiseStatus>),
    New(JobTicket, watch::Receiver<DenoiseStatus>),
    Busy,
}

#[derive(Clone, Default)]
pub(super) struct JobTable {
    inner: Arc<Mutex<Inner>>,
}

impl JobTable {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn admit(&self, key: &PlaneKey, owner_limit: Option<usize>) -> Admission {
        let mut inner = self.lock();
        inner.expire(Instant::now());
        if let Some(entry) = inner.entries.get(key)
            && !matches!(*entry.rx.borrow(), DenoiseStatus::Failed { .. })
        {
            return Admission::Existing(entry.rx.clone());
        }
        if owner_limit.is_some_and(|limit| inner.pending_for(key.owner) >= limit) {
            return Admission::Busy;
        }
        inner.next_id += 1;
        let id = inner.next_id;
        let (tx, rx) = watch::channel(DenoiseStatus::Queued);
        inner.entries.insert(
            key.clone(),
            Entry {
                id,
                rx: rx.clone(),
                settled_at: None,
            },
        );
        let ticket = JobTicket {
            table: self.clone(),
            key: key.clone(),
            id,
            tx,
            finished: false,
        };
        Admission::New(ticket, rx)
    }

    pub(super) fn state(&self, key: &PlaneKey) -> Option<DenoiseStatus> {
        let mut inner = self.lock();
        inner.expire(Instant::now());
        inner
            .entries
            .get(key)
            .map(|entry| entry.rx.borrow().clone())
    }

    pub(super) fn drop_owner(&self, owner: Uuid) {
        self.lock().entries.retain(|key, _| key.owner != owner);
    }

    pub(super) fn clear(&self) {
        self.lock().entries.clear();
    }

    fn settle(
        &self,
        key: &PlaneKey,
        id: u64,
        tx: &watch::Sender<DenoiseStatus>,
        state: DenoiseStatus,
    ) {
        let mut inner = self.lock();
        let ready = matches!(state, DenoiseStatus::Ready);
        if inner.entries.get(key).is_some_and(|entry| entry.id == id) {
            if ready {
                inner.entries.remove(key);
            } else if let Some(entry) = inner.entries.get_mut(key) {
                entry.settled_at = Some(Instant::now());
            }
        }
        tx.send_replace(state);
    }

    #[cfg(test)]
    fn expire_at(&self, now: Instant) {
        self.lock().expire(now);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.lock().entries.len()
    }
}

pub(super) struct JobTicket {
    table: JobTable,
    key: PlaneKey,
    id: u64,
    tx: watch::Sender<DenoiseStatus>,
    finished: bool,
}

impl JobTicket {
    pub(super) fn progress(&self) -> watch::Sender<DenoiseStatus> {
        self.tx.clone()
    }

    pub(super) fn finish(&mut self, state: DenoiseStatus) {
        self.finished = true;
        self.table.settle(&self.key, self.id, &self.tx, state);
    }
}

impl Drop for JobTicket {
    fn drop(&mut self) {
        if !self.finished {
            self.finish(DenoiseStatus::Failed {
                error: DenoiseError::stopped("the AI denoise bake ended without a result"),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(owner: u128, source: u128) -> PlaneKey {
        PlaneKey {
            server_epoch: 1,
            owner: Uuid::from_u128(owner),
            source: Uuid::from_u128(source),
            model: "nind_utnet".into(),
        }
    }

    fn admit_new(table: &JobTable, key: &PlaneKey) -> JobTicket {
        match table.admit(key, Some(MAX_PENDING_PER_OWNER)) {
            Admission::New(ticket, _) => ticket,
            Admission::Existing(_) => panic!("expected a new job, got an existing one"),
            Admission::Busy => panic!("expected a new job, got busy"),
        }
    }

    #[test]
    fn an_owner_cannot_queue_more_than_the_limit() {
        let table = JobTable::default();
        let mut tickets: Vec<JobTicket> = (0..MAX_PENDING_PER_OWNER as u128)
            .map(|source| admit_new(&table, &key(1, source)))
            .collect();
        let extra = key(1, 99);
        assert!(matches!(
            table.admit(&extra, Some(MAX_PENDING_PER_OWNER)),
            Admission::Busy
        ));
        assert!(matches!(table.admit(&extra, None), Admission::New(..)));
        admit_new(&table, &key(2, 0)).finish(DenoiseStatus::Ready);
        assert!(matches!(
            table.admit(&key(1, 0), Some(MAX_PENDING_PER_OWNER)),
            Admission::Existing(_)
        ));
        tickets[0].finish(DenoiseStatus::Unsupported);
        let _next = admit_new(&table, &key(1, 100));
        assert!(matches!(
            table.admit(&key(1, 101), Some(MAX_PENDING_PER_OWNER)),
            Admission::Busy
        ));
    }

    #[test]
    fn settled_failures_expire_and_ready_jobs_leave_at_once() {
        let table = JobTable::default();
        admit_new(&table, &key(1, 1)).finish(DenoiseStatus::Ready);
        assert_eq!(table.len(), 0);
        admit_new(&table, &key(1, 2)).finish(DenoiseStatus::Unsupported);
        admit_new(&table, &key(1, 3)).finish(DenoiseStatus::Failed {
            error: DenoiseError::Purged,
        });
        let running = admit_new(&table, &key(1, 4));
        assert_eq!(table.len(), 3);
        table.expire_at(Instant::now() + SETTLED_TTL / 2);
        assert_eq!(table.len(), 3);
        table.expire_at(Instant::now() + SETTLED_TTL);
        assert_eq!(table.len(), 1);
        assert!(table.state(&key(1, 4)).is_some());
        drop(running);
        assert!(matches!(
            table.state(&key(1, 4)),
            Some(DenoiseStatus::Failed {
                error: DenoiseError::Internal { .. }
            })
        ));
    }

    #[test]
    fn a_stale_job_cannot_settle_its_retry() {
        let table = JobTable::default();
        let k = key(1, 1);
        let mut first = admit_new(&table, &k);
        table.drop_owner(k.owner);
        let _retry = admit_new(&table, &k);
        first.finish(DenoiseStatus::Ready);
        assert!(matches!(table.state(&k), Some(DenoiseStatus::Queued)));
    }
}
