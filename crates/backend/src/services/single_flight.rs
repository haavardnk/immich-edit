use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tokio::sync::watch;

type Flights<K, V> = Arc<Mutex<HashMap<K, watch::Receiver<Option<V>>>>>;

pub struct SingleFlight<K, V> {
    flights: Flights<K, V>,
}

impl<K, V> Clone for SingleFlight<K, V> {
    fn clone(&self) -> Self {
        Self {
            flights: self.flights.clone(),
        }
    }
}

impl<K, V> Default for SingleFlight<K, V> {
    fn default() -> Self {
        Self {
            flights: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct Release<K: Hash + Eq, V> {
    flights: Flights<K, V>,
    key: K,
}

impl<K: Hash + Eq, V> Drop for Release<K, V> {
    fn drop(&mut self) {
        lock(&self.flights).remove(&self.key);
    }
}

impl<K, V> SingleFlight<K, V>
where
    K: Hash + Eq + Clone + Send + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub async fn run<F, Fut>(&self, key: K, work: F) -> Option<V>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V> + Send + 'static,
    {
        let mut flight = self.join(key, work);
        loop {
            let current = flight.borrow_and_update().clone();
            if current.is_some() {
                return current;
            }
            if flight.changed().await.is_err() {
                return None;
            }
        }
    }

    fn join<F, Fut>(&self, key: K, work: F) -> watch::Receiver<Option<V>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V> + Send + 'static,
    {
        let (done, flight) = {
            let mut flights = lock(&self.flights);
            if let Some(flight) = flights.get(&key) {
                return flight.clone();
            }
            let (done, flight) = watch::channel(None);
            flights.insert(key.clone(), flight.clone());
            (done, flight)
        };
        let release = Release {
            flights: self.flights.clone(),
            key,
        };
        let task = work();
        tokio::spawn(async move {
            let value = task.await;
            drop(release);
            let _ = done.send(Some(value));
        });
        flight
    }
}

fn lock<K, V>(flights: &Flights<K, V>) -> MutexGuard<'_, HashMap<K, watch::Receiver<Option<V>>>> {
    flights.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use tokio::sync::{Notify, oneshot};

    use super::*;

    #[tokio::test]
    async fn concurrent_callers_share_one_run() {
        let flights: SingleFlight<&str, usize> = SingleFlight::default();
        let runs = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Notify::new());
        let work = || {
            let runs = runs.clone();
            let gate = gate.clone();
            move || async move {
                gate.notified().await;
                runs.fetch_add(1, Ordering::SeqCst) + 7
            }
        };
        let first = flights.run("key", work());
        let second = flights.run("key", work());
        let open = async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            gate.notify_waiters();
        };
        let (first, second, ()) = tokio::join!(first, second, open);
        let shared = (first, second, runs.load(Ordering::SeqCst));
        if shared != (Some(7), Some(7), 1) {
            panic!("concurrent callers saw {shared:?}");
        }
    }

    #[tokio::test]
    async fn a_dropped_caller_still_finishes_the_work() {
        let flights: SingleFlight<&str, usize> = SingleFlight::default();
        let (finished, observed) = oneshot::channel();
        let caller = tokio::spawn({
            let flights = flights.clone();
            async move {
                flights
                    .run("key", move || async move {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        let _ = finished.send(());
                        1
                    })
                    .await
            }
        });
        tokio::time::sleep(Duration::from_millis(5)).await;
        caller.abort();
        let completed = tokio::time::timeout(Duration::from_secs(5), observed).await;
        if !matches!(completed, Ok(Ok(()))) {
            panic!("work stopped with its caller: {completed:?}");
        }
    }

    #[tokio::test]
    async fn a_finished_key_runs_again() {
        let flights: SingleFlight<&str, usize> = SingleFlight::default();
        let first = flights.run("key", || async { 1 }).await;
        let second = flights.run("key", || async { 2 }).await;
        if (first, second) != (Some(1), Some(2)) {
            panic!("sequential runs gave {first:?} then {second:?}");
        }
    }

    async fn fails() -> usize {
        panic!("work failed on purpose")
    }

    #[tokio::test]
    async fn a_panicking_run_releases_its_key_and_waiters() {
        let flights: SingleFlight<&str, usize> = SingleFlight::default();
        let failed = flights.run("key", fails).await;
        let retried = flights.run("key", || async { 3 }).await;
        if (failed, retried) != (None, Some(3)) {
            panic!("panicked run gave {failed:?} then {retried:?}");
        }
    }
}
