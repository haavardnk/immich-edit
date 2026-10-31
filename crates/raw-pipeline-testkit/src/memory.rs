use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

const MIN_COVERAGE: f64 = 0.95;
const MAX_OVERESTIMATE: f64 = 2.0;

static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
static MEASURING: Mutex<()> = Mutex::new(());

pub struct Counting;

fn grow(bytes: usize) {
    let live = LIVE.fetch_add(bytes as u64, Ordering::Relaxed) + bytes as u64;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

fn shrink(bytes: usize) {
    LIVE.fetch_sub(bytes as u64, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            grow(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            grow(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        shrink(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        grow(size);
        let moved = unsafe { System.realloc(ptr, layout, size) };
        if moved.is_null() {
            shrink(size);
        } else {
            shrink(layout.size());
        }
        moved
    }
}

pub fn measuring() -> MutexGuard<'static, ()> {
    MEASURING.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn heap_peak(work: impl FnOnce()) -> u64 {
    let live = LIVE.load(Ordering::SeqCst);
    PEAK.store(live, Ordering::SeqCst);
    work();
    PEAK.load(Ordering::SeqCst) - live
}

pub fn check(name: &str, measured: u64, estimate: u64, failures: &mut Vec<String>) {
    let ratio = estimate as f64 / measured as f64;
    eprintln!(
        "{name}: measured {} MiB, estimate {} MiB, ratio {ratio:.3}",
        measured >> 20,
        estimate >> 20
    );
    if !(MIN_COVERAGE..=MAX_OVERESTIMATE).contains(&ratio) {
        failures.push(format!("{name} at {ratio:.3}x"));
    }
}

pub fn finish(failures: Vec<String>) {
    if !failures.is_empty() {
        panic!("estimates outside {MIN_COVERAGE}..={MAX_OVERESTIMATE}: {failures:?}");
    }
}
