use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Default)]
pub struct CancelTracker {
    latest: Arc<AtomicU64>,
}

impl CancelTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&self) -> CancelToken {
        let g = self.latest.fetch_add(1, Ordering::Relaxed) + 1;
        CancelToken {
            tracker: self.latest.clone(),
            generation: g,
        }
    }
}

#[derive(Clone)]
pub struct CancelToken {
    tracker: Arc<AtomicU64>,
    generation: u64,
}

impl CancelToken {
    pub fn is_cancelled(&self) -> bool {
        self.tracker.load(Ordering::Relaxed) != self.generation
    }

    pub fn check(&self) -> crate::PipelineResult<()> {
        if self.is_cancelled() {
            Err(crate::PipelineError::Cancelled)
        } else {
            Ok(())
        }
    }
}

pub fn check(token: Option<&CancelToken>) -> crate::PipelineResult<()> {
    match token {
        Some(t) => t.check(),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_token_not_cancelled() {
        let t = CancelTracker::new();
        let tok = t.next();
        if tok.is_cancelled() {
            panic!("fresh token should not be cancelled");
        }
    }

    #[test]
    fn newer_token_cancels_prior() {
        let t = CancelTracker::new();
        let a = t.next();
        let _b = t.next();
        if !a.is_cancelled() {
            panic!("prior token should be cancelled");
        }
    }
}
