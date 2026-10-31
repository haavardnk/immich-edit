use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::{Admission, HostMemory, Retained};
use crate::PipelineError;
use crate::cancel::CancelTracker;

static IDLE: Cache = Cache(AtomicU64::new(0));

#[test]
fn a_render_over_the_budget_is_refused_and_a_smaller_one_admitted() {
    let admission = Admission::new("memory", Some(100));
    if !matches!(
        admission.admit(101, &IDLE, None),
        Err(PipelineError::OverBudget(_))
    ) {
        panic!("a render over the budget was not refused");
    }
    if let Err(e) = admission.admit(100, &IDLE, None) {
        panic!("a render at the budget was refused: {e}");
    }
}

#[test]
fn retained_memory_is_trimmed_by_the_excess() {
    let admission = Admission::new("memory", Some(100));
    let cache = Cache(AtomicU64::new(50));
    if let Err(e) = admission.admit(80, &cache, None) {
        panic!("a render that fits after trimming was refused: {e}");
    }
    if cache.retained_bytes() != 20 {
        panic!("{} bytes left, expected 20", cache.retained_bytes());
    }
}

#[test]
fn a_render_waits_for_one_in_flight_and_runs_when_it_finishes() {
    let admission = Admission::new("memory", Some(100));
    let first = admission.admit(60, &IDLE, None).unwrap();
    let (sent, received) = mpsc::channel();
    thread::scope(|scope| {
        scope.spawn(|| {
            let second = admission.admit(60, &IDLE, None);
            sent.send(second.is_ok()).unwrap();
        });
        if received.recv_timeout(Duration::from_millis(100)).is_ok() {
            panic!("a render was admitted while the budget was taken");
        }
        drop(first);
        if received.recv_timeout(Duration::from_secs(5)) != Ok(true) {
            panic!("the waiting render was not admitted after the first finished");
        }
    });
}

#[test]
fn a_waiting_render_is_cancelled() {
    let admission = Admission::new("memory", Some(100));
    let _first = admission.admit(60, &IDLE, None).unwrap();
    let token = CancelTracker::new().next();
    thread::scope(|scope| {
        let waiting = scope.spawn(|| admission.admit(60, &IDLE, Some(&token)).map(drop));
        thread::sleep(Duration::from_millis(50));
        token.cancel();
        if !matches!(waiting.join().unwrap(), Err(PipelineError::Cancelled)) {
            panic!("a cancelled render kept waiting for memory");
        }
    });
}

#[test]
fn host_memory_clones_share_one_gate() {
    let host = HostMemory::new(Some(100));
    let shared = host.clone();
    let _first = host.admit(60, None).unwrap();
    let token = CancelTracker::new().next();
    thread::scope(|scope| {
        let waiting = scope.spawn(|| shared.admit(60, Some(&token)).map(drop));
        thread::sleep(Duration::from_millis(50));
        token.cancel();
        if !matches!(waiting.join().unwrap(), Err(PipelineError::Cancelled)) {
            panic!("a clone admitted a render past memory another clone holds");
        }
    });
}

struct Cache(AtomicU64);

impl Retained for Cache {
    fn retained_bytes(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    fn trim(&self, excess: u64) {
        let held = self.retained_bytes();
        self.0.store(held.saturating_sub(excess), Ordering::SeqCst);
    }
}

#[test]
fn host_memory_trims_every_registered_cache_until_a_render_fits() {
    let host = HostMemory::new(Some(100));
    let cpu = Arc::new(Cache(AtomicU64::new(30)));
    let gpu = Arc::new(Cache(AtomicU64::new(40)));
    host.register(cpu.clone());
    host.register(gpu.clone());
    if let Err(e) = host.admit(90, None) {
        panic!("a render that fits after trimming both caches was refused: {e}");
    }
    let left = (cpu.retained_bytes(), gpu.retained_bytes());
    if left.0 + left.1 > 10 {
        panic!("trimming left {left:?} beside a 90 byte render in a 100 byte budget");
    }
}

#[test]
fn registering_a_cache_does_not_keep_it_alive() {
    let host = HostMemory::new(Some(100));
    let cache = Arc::new(Cache(AtomicU64::new(100)));
    host.register(cache.clone());
    let weak = Arc::downgrade(&cache);
    drop(cache);
    if weak.upgrade().is_some() {
        panic!("registering kept a cache alive");
    }
}

#[test]
fn an_out_of_memory_lowers_the_budget_to_what_was_in_use() {
    let unlimited = Admission::new("memory", None);
    let held = unlimited.admit(800, &IDLE, None).unwrap();
    unlimited.lower(200);
    drop(held);
    if unlimited.budget() != Some(900) {
        panic!("learned {:?}, expected 900", unlimited.budget());
    }
    unlimited.lower(10_000);
    if unlimited.budget() != Some(900) {
        panic!(
            "a later failure raised the budget to {:?}",
            unlimited.budget()
        );
    }
}
