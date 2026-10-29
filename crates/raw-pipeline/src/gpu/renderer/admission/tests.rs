use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::{Admission, Retention};
use crate::PipelineError;
use crate::cancel::CancelTracker;

fn nothing() -> u64 {
    0
}

fn ignore(_: u64) {}

const IDLE: Retention<'static> = Retention {
    bytes: &nothing,
    trim: &ignore,
};

#[test]
fn a_render_over_the_budget_is_refused_and_a_smaller_one_admitted() {
    let admission = Admission::new(Some(100));
    if !matches!(
        admission.admit(101, IDLE, None),
        Err(PipelineError::Unsupported(_))
    ) {
        panic!("a render over the budget was not refused");
    }
    if let Err(e) = admission.admit(100, IDLE, None) {
        panic!("a render at the budget was refused: {e}");
    }
}

#[test]
fn retained_memory_is_trimmed_by_the_excess() {
    let admission = Admission::new(Some(100));
    let retained = AtomicU64::new(50);
    let trimmed = AtomicU64::new(0);
    let bytes = || retained.load(Ordering::SeqCst);
    let trim = |excess: u64| {
        trimmed.store(excess, Ordering::SeqCst);
        retained.fetch_sub(excess, Ordering::SeqCst);
    };
    let retention = Retention {
        bytes: &bytes,
        trim: &trim,
    };
    if let Err(e) = admission.admit(80, retention, None) {
        panic!("a render that fits after trimming was refused: {e}");
    }
    if trimmed.load(Ordering::SeqCst) != 30 {
        panic!(
            "trimmed {} bytes, expected 30",
            trimmed.load(Ordering::SeqCst)
        );
    }
}

#[test]
fn a_render_waits_for_one_in_flight_and_runs_when_it_finishes() {
    let admission = Admission::new(Some(100));
    let first = admission.admit(60, IDLE, None).unwrap();
    let (sent, received) = mpsc::channel();
    thread::scope(|scope| {
        scope.spawn(|| {
            let second = admission.admit(60, IDLE, None);
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
    let admission = Admission::new(Some(100));
    let _first = admission.admit(60, IDLE, None).unwrap();
    let token = CancelTracker::new().next();
    thread::scope(|scope| {
        let waiting = scope.spawn(|| admission.admit(60, IDLE, Some(&token)).map(drop));
        thread::sleep(Duration::from_millis(50));
        token.cancel();
        if !matches!(waiting.join().unwrap(), Err(PipelineError::Cancelled)) {
            panic!("a cancelled render kept waiting for gpu memory");
        }
    });
}

#[test]
fn an_out_of_memory_lowers_the_budget_to_what_was_in_use() {
    let unlimited = Admission::new(None);
    let held = unlimited.admit(800, IDLE, None).unwrap();
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
