use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use raw_pipeline::timing::StageTiming;

const SAMPLE_CAP: usize = 256;

#[derive(Clone, Default)]
pub struct RenderTelemetry {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    cpu: RwLock<Samples>,
    gpu: RwLock<Samples>,
    stages: RwLock<Vec<StageSamples>>,
    fetch: RwLock<Samples>,
    decode: RwLock<Samples>,
    frame_hits: AtomicU64,
    frame_misses: AtomicU64,
    request_timeouts: AtomicU64,
}

#[derive(Default)]
struct Samples(VecDeque<u64>);

impl Samples {
    fn push(&mut self, dur: Duration) {
        if self.0.len() == SAMPLE_CAP {
            self.0.pop_front();
        }
        self.0
            .push_back(dur.as_micros().min(u128::from(u64::MAX)) as u64);
    }

    fn stats(&self) -> LatencyStats {
        summarize(&self.0)
    }
}

struct StageSamples {
    kind: RendererKind,
    stage: &'static str,
    wall: Samples,
    gpu: Samples,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Cpu,
    Gpu,
}

impl RendererKind {
    pub fn from_label(label: &str) -> Self {
        if label == "gpu" { Self::Gpu } else { Self::Cpu }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
        }
    }
}

impl RenderTelemetry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, kind: RendererKind, total: Duration, stages: &[StageTiming]) {
        let buf = match kind {
            RendererKind::Cpu => &self.inner.cpu,
            RendererKind::Gpu => &self.inner.gpu,
        };
        buf.write().unwrap().push(total);
        let mut stages_buf = self.inner.stages.write().unwrap();
        for s in stages {
            let index = match stages_buf
                .iter()
                .position(|e| e.kind == kind && e.stage == s.stage)
            {
                Some(i) => i,
                None => {
                    stages_buf.push(StageSamples {
                        kind,
                        stage: s.stage,
                        wall: Samples::default(),
                        gpu: Samples::default(),
                    });
                    stages_buf.len() - 1
                }
            };
            let entry = &mut stages_buf[index];
            entry.wall.push(s.wall);
            if let Some(gpu) = s.gpu {
                entry.gpu.push(gpu);
            }
        }
    }

    pub fn record_frame_hit(&self) {
        self.inner.frame_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_frame_miss(&self) {
        self.inner.frame_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_frame_load(&self, fetch: Duration, decode: Duration) {
        self.inner.fetch.write().unwrap().push(fetch);
        self.inner.decode.write().unwrap().push(decode);
    }

    pub fn record_request_timeout(&self) {
        self.inner.request_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        let stages = self
            .inner
            .stages
            .read()
            .unwrap()
            .iter()
            .map(|s| StageStats {
                renderer: s.kind.as_str(),
                stage: s.stage,
                wall: s.wall.stats(),
                gpu: (!s.gpu.0.is_empty()).then(|| s.gpu.stats()),
            })
            .collect();
        TelemetrySnapshot {
            render_latency: RenderLatency {
                cpu: self.inner.cpu.read().unwrap().stats(),
                gpu: self.inner.gpu.read().unwrap().stats(),
            },
            stages,
            frames: FrameStats {
                fetch: self.inner.fetch.read().unwrap().stats(),
                decode: self.inner.decode.read().unwrap().stats(),
                cache_hits: self.inner.frame_hits.load(Ordering::Relaxed),
                cache_misses: self.inner.frame_misses.load(Ordering::Relaxed),
            },
            request_timeouts: self.inner.request_timeouts.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct LatencyStats {
    pub count: usize,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct RenderLatency {
    pub cpu: LatencyStats,
    pub gpu: LatencyStats,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StageStats {
    pub renderer: &'static str,
    pub stage: &'static str,
    pub wall: LatencyStats,
    pub gpu: Option<LatencyStats>,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct FrameStats {
    pub fetch: LatencyStats,
    pub decode: LatencyStats,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TelemetrySnapshot {
    pub render_latency: RenderLatency,
    pub stages: Vec<StageStats>,
    pub frames: FrameStats,
    pub request_timeouts: u64,
}

fn summarize(buf: &VecDeque<u64>) -> LatencyStats {
    if buf.is_empty() {
        return LatencyStats::default();
    }
    let mut v: Vec<u64> = buf.iter().copied().collect();
    v.sort_unstable();
    let n = v.len();
    LatencyStats {
        count: n,
        p50_us: percentile(&v, 0.50),
        p95_us: percentile(&v, 0.95),
        p99_us: percentile(&v, 0.99),
        max_us: *v.last().unwrap(),
    }
}

fn percentile(sorted: &[u64], q: f64) -> u64 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let idx = ((q * n as f64).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(stage: &'static str, wall_ms: u64, gpu_ms: Option<u64>) -> StageTiming {
        StageTiming {
            stage,
            wall: Duration::from_millis(wall_ms),
            gpu: gpu_ms.map(Duration::from_millis),
        }
    }

    #[test]
    fn stages_are_kept_per_renderer_with_their_own_percentiles() {
        let t = RenderTelemetry::new();
        for ms in 1..=20 {
            t.record(
                RendererKind::Gpu,
                Duration::from_millis(ms * 2),
                &[
                    stage("demosaic", ms, Some(ms / 2)),
                    stage("encode", 3, None),
                ],
            );
        }
        t.record(
            RendererKind::Cpu,
            Duration::from_millis(90),
            &[stage("demosaic", 60, None)],
        );

        let snap = t.snapshot();
        let order: Vec<(&str, &str)> = snap.stages.iter().map(|s| (s.renderer, s.stage)).collect();
        if order != [("gpu", "demosaic"), ("gpu", "encode"), ("cpu", "demosaic")] {
            panic!("stages should keep pipeline order, got {order:?}");
        }
        let find = |renderer: &str, name: &str| {
            snap.stages
                .iter()
                .find(|s| s.renderer == renderer && s.stage == name)
                .cloned()
        };
        let Some(gpu_demosaic) = find("gpu", "demosaic") else {
            panic!("missing gpu demosaic in {:?}", snap.stages);
        };
        if gpu_demosaic.wall.count != 20
            || gpu_demosaic.wall.p50_us != 10_000
            || gpu_demosaic.wall.p95_us != 19_000
        {
            panic!("gpu demosaic wall stats wrong: {:?}", gpu_demosaic.wall);
        }
        if gpu_demosaic.gpu.map(|g| g.p95_us) != Some(9_000) {
            panic!("gpu demosaic device time wrong: {:?}", gpu_demosaic.gpu);
        }
        match find("gpu", "encode") {
            Some(s) if s.gpu.is_none() && s.wall.p50_us == 3_000 => {}
            other => panic!("wall-only encode stage wrong: {other:?}"),
        }
        match find("cpu", "demosaic") {
            Some(s) if s.wall.count == 1 && s.wall.max_us == 60_000 => {}
            other => panic!("cpu demosaic mixed with gpu samples: {other:?}"),
        }
        if snap.render_latency.gpu.count != 20 || snap.render_latency.cpu.count != 1 {
            panic!("totals wrong: {:?}", snap.render_latency);
        }
    }

    #[test]
    fn frame_loads_and_timeouts_are_counted() {
        let t = RenderTelemetry::new();
        t.record_frame_miss();
        t.record_frame_load(Duration::from_millis(400), Duration::from_millis(1_200));
        t.record_frame_hit();
        t.record_frame_hit();
        t.record_request_timeout();

        let snap = t.snapshot();
        if snap.frames.cache_hits != 2 || snap.frames.cache_misses != 1 {
            panic!("frame cache counts wrong: {:?}", snap.frames);
        }
        if snap.frames.fetch.max_us != 400_000 || snap.frames.decode.max_us != 1_200_000 {
            panic!("frame load timings wrong: {:?}", snap.frames);
        }
        if snap.request_timeouts != 1 {
            panic!("timeouts wrong: {}", snap.request_timeouts);
        }
    }

    #[test]
    fn samples_keep_only_the_most_recent_window() {
        let t = RenderTelemetry::new();
        for ms in 0..(SAMPLE_CAP as u64 + 10) {
            t.record(RendererKind::Cpu, Duration::from_millis(ms), &[]);
        }
        let cpu = t.snapshot().render_latency.cpu;
        if cpu.count != SAMPLE_CAP || cpu.max_us != (SAMPLE_CAP as u64 + 9) * 1_000 {
            panic!("window not capped: {cpu:?}");
        }
    }
}
