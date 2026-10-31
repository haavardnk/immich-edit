use std::fmt;
use std::sync::{Arc, Weak};

use parking_lot::Mutex;

use super::{Admission, Admitted, Retained};
use crate::PipelineResult;
use crate::cancel::CancelToken;

#[derive(Clone)]
pub struct HostMemory {
    admission: Arc<Admission>,
    sources: Arc<Sources>,
}

impl HostMemory {
    pub fn new(budget: Option<u64>) -> Self {
        Self {
            admission: Arc::new(Admission::new("memory", budget)),
            sources: Arc::default(),
        }
    }

    pub fn budget(&self) -> Option<u64> {
        self.admission.budget()
    }

    pub(crate) fn register(&self, source: Arc<dyn Retained>) {
        let mut sources = self.sources.lock();
        sources.retain(|source| source.strong_count() > 0);
        sources.push(Arc::downgrade(&source));
    }

    pub(crate) fn admit(
        &self,
        bytes: u64,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        self.admission.admit(bytes, &*self.sources, cancel)
    }
}

type Sources = Mutex<Vec<Weak<dyn Retained>>>;

fn live(sources: &Sources) -> Vec<Arc<dyn Retained>> {
    sources.lock().iter().filter_map(Weak::upgrade).collect()
}

impl Retained for Sources {
    fn retained_bytes(&self) -> u64 {
        live(self)
            .iter()
            .map(|source| source.retained_bytes())
            .sum()
    }

    fn trim(&self, excess: u64) {
        let mut left = excess;
        for source in live(self) {
            if left == 0 {
                return;
            }
            let before = source.retained_bytes();
            source.trim(left);
            left = left.saturating_sub(before.saturating_sub(source.retained_bytes()));
        }
    }
}

impl Default for HostMemory {
    fn default() -> Self {
        Self::new(None)
    }
}

impl fmt::Debug for HostMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostMemory")
            .field("budget", &self.budget())
            .finish()
    }
}
