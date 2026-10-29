use std::time::Duration;

use parking_lot::{Condvar, Mutex};

use super::GpuRenderer;
use crate::cancel::CancelToken;
use crate::{PipelineError, PipelineResult};

#[cfg(test)]
mod tests;

const CANCEL_POLL: Duration = Duration::from_millis(20);
const LEARNED_PERCENT: u64 = 90;
const MIB: u64 = 1024 * 1024;

pub(super) struct Admission {
    state: Mutex<State>,
    released: Condvar,
}

struct State {
    budget: Option<u64>,
    in_flight: u64,
}

struct Retention<'a> {
    bytes: &'a dyn Fn() -> u64,
    trim: &'a dyn Fn(u64),
}

pub(super) struct Admitted<'a> {
    admission: &'a Admission,
    bytes: u64,
}

impl Admission {
    pub fn new(budget: Option<u64>) -> Self {
        Self {
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

    fn admit(
        &self,
        bytes: u64,
        retention: Retention<'_>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        let mut state = self.state.lock();
        while let Some(budget) = state.budget {
            crate::cancel::check(cancel)?;
            if bytes > budget {
                return Err(PipelineError::Unsupported(format!(
                    "render needs {} MiB of gpu memory, budget {} MiB",
                    bytes.div_ceil(MIB),
                    budget / MIB
                )));
            }
            let excess = (state.in_flight + (retention.bytes)() + bytes).saturating_sub(budget);
            if excess > 0 {
                (retention.trim)(excess);
            }
            if state.in_flight == 0 || state.in_flight + (retention.bytes)() + bytes <= budget {
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

impl GpuRenderer {
    pub(super) fn admit(
        &self,
        bytes: u64,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<Admitted<'_>> {
        let retention = Retention {
            bytes: &|| self.retained_bytes(),
            trim: &|excess| self.trim_retained(excess),
        };
        self.admission.admit(bytes, retention, cancel)
    }
}

impl Drop for Admitted<'_> {
    fn drop(&mut self) {
        self.admission.state.lock().in_flight -= self.bytes;
        self.admission.released.notify_all();
    }
}
