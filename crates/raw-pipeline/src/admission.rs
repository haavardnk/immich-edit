use std::time::Duration;

use parking_lot::{Condvar, Mutex};

use crate::cancel::CancelToken;
use crate::{PipelineError, PipelineResult};

mod host;
mod idle;
#[cfg(test)]
mod tests;

pub use host::HostMemory;
pub(crate) use idle::{evict_idle, idle_bytes, unshared};

const CANCEL_POLL: Duration = Duration::from_millis(20);
const LEARNED_PERCENT: u64 = 90;
const MIB: u64 = 1024 * 1024;

pub(crate) struct Admission {
    memory: &'static str,
    state: Mutex<State>,
    released: Condvar,
}

struct State {
    budget: Option<u64>,
    in_flight: u64,
}

pub(crate) trait Retained: Send + Sync {
    fn retained_bytes(&self) -> u64;
    fn trim(&self, excess: u64);
}

pub(crate) struct Admitted<'a> {
    admission: &'a Admission,
    bytes: u64,
}

pub(crate) fn over_budget(task: &str, bytes: u64, memory: &str, budget: u64) -> PipelineError {
    PipelineError::OverBudget(format!(
        "{task} needs {} MiB of {memory}, budget {} MiB",
        bytes.div_ceil(MIB),
        budget / MIB
    ))
}

impl Admission {
    pub fn new(memory: &'static str, budget: Option<u64>) -> Self {
        Self {
            memory,
            state: Mutex::new(State {
                budget,
                in_flight: 0,
            }),
            released: Condvar::new(),
        }
    }

    pub fn budget(&self) -> Option<u64> {
        self.state.lock().budget
    }

    pub fn admit(
        &self,
        bytes: u64,
        retained: &dyn Retained,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        let mut state = self.state.lock();
        while let Some(budget) = state.budget {
            crate::cancel::check(cancel)?;
            if bytes > budget {
                return Err(over_budget("render", bytes, self.memory, budget));
            }
            let excess =
                (state.in_flight + retained.retained_bytes() + bytes).saturating_sub(budget);
            if excess > 0 {
                retained.trim(excess);
            }
            if state.in_flight == 0 || state.in_flight + retained.retained_bytes() + bytes <= budget
            {
                break;
            }
            self.released.wait_for(&mut state, CANCEL_POLL);
        }
        state.in_flight += bytes;
        Ok(Admitted {
            admission: self,
            bytes,
        })
    }

    pub fn lower(&self, retained: u64) {
        let mut state = self.state.lock();
        let learned = (state.in_flight + retained) * LEARNED_PERCENT / 100;
        state.budget = Some(state.budget.map_or(learned, |budget| budget.min(learned)));
    }
}

impl Drop for Admitted<'_> {
    fn drop(&mut self) {
        self.admission.state.lock().in_flight -= self.bytes;
        self.admission.released.notify_all();
    }
}
