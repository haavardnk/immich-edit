use std::time::Duration;

use parking_lot::Mutex;
use web_time::Instant;

pub const DEMOSAIC: &str = "demosaic";
pub const LENS: &str = "lens";
pub const WB_PREPARE: &str = "wb_prepare";
pub const RETOUCH: &str = "retouch";
pub const NOISE_REDUCTION: &str = "noise_reduction";
pub const CAPTURE_SHARPEN: &str = "capture_sharpen";
pub const PREVIEW_RESAMPLE: &str = "preview_resample";
pub const DEHAZE: &str = "dehaze";
pub const PRESENCE: &str = "presence";
pub const SHADOWS: &str = "shadows";
pub const POINTWISE: &str = "pointwise";
pub const MASKS: &str = "masks";
pub const RESAMPLE: &str = "resample";
pub const DISPLAY: &str = "display";
pub const OUTPUT: &str = "output";
pub const FINISH: &str = "finish";
pub const EXPORT_FINISH: &str = "export_finish";
pub const READBACK: &str = "readback";
pub const HISTOGRAM: &str = "histogram";
pub const SCOPES: &str = "scopes";
pub const ENCODE: &str = "encode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageTiming {
    pub stage: &'static str,
    pub wall: Duration,
    pub gpu: Option<Duration>,
}

#[derive(Default)]
pub struct StageClock {
    samples: Mutex<Vec<StageTiming>>,
}

impl StageClock {
    pub fn time<T>(&self, stage: &'static str, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let out = f();
        self.add_wall(stage, start.elapsed());
        out
    }

    pub fn add_wall(&self, stage: &'static str, wall: Duration) {
        self.entry(stage, |t| t.wall += wall);
    }

    pub fn add_gpu(&self, stage: &'static str, gpu: Duration) {
        self.entry(stage, |t| t.gpu = Some(t.gpu.unwrap_or_default() + gpu));
    }

    pub fn finish(self) -> Vec<StageTiming> {
        self.samples.into_inner()
    }

    fn entry(&self, stage: &'static str, update: impl FnOnce(&mut StageTiming)) {
        let mut samples = self.samples.lock();
        match samples.iter_mut().find(|t| t.stage == stage) {
            Some(t) => update(t),
            None => {
                let mut t = StageTiming {
                    stage,
                    wall: Duration::ZERO,
                    gpu: None,
                };
                update(&mut t);
                samples.push(t);
            }
        }
    }
}

pub fn cpu_op_stage(op_id: &'static str) -> &'static str {
    match op_id {
        "luma_nr" | "color_nr" => NOISE_REDUCTION,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_stages_sum_in_first_seen_order() {
        let clock = StageClock::default();
        clock.add_wall(DEMOSAIC, Duration::from_millis(5));
        clock.add_wall(ENCODE, Duration::from_millis(2));
        clock.add_wall(DEMOSAIC, Duration::from_millis(3));
        clock.add_gpu(DEMOSAIC, Duration::from_millis(1));
        clock.add_gpu(DEMOSAIC, Duration::from_millis(4));

        let expected = vec![
            StageTiming {
                stage: DEMOSAIC,
                wall: Duration::from_millis(8),
                gpu: Some(Duration::from_millis(5)),
            },
            StageTiming {
                stage: ENCODE,
                wall: Duration::from_millis(2),
                gpu: None,
            },
        ];
        let got = clock.finish();
        if got != expected {
            panic!("expected {expected:?}, got {got:?}");
        }
    }

    #[test]
    fn time_returns_the_closure_value_and_records_the_stage() {
        let clock = StageClock::default();
        let v = clock.time(SCOPES, || 7);
        let samples = clock.finish();
        if v != 7 {
            panic!("closure value lost: {v}");
        }
        if samples.len() != 1 || samples[0].stage != SCOPES || samples[0].gpu.is_some() {
            panic!("expected one wall-only scopes sample, got {samples:?}");
        }
    }
}
