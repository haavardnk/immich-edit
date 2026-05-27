use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

const SAMPLE_CAP: usize = 256;

#[derive(Clone, Default)]
pub struct RenderTelemetry {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    cpu: RwLock<VecDeque<u64>>,
    gpu: RwLock<VecDeque<u64>>,
}

#[derive(Debug, Clone, Copy)]
pub enum RendererKind {
    Cpu,
    Gpu,
}

impl RenderTelemetry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, kind: RendererKind, dur: Duration) {
        let micros = dur.as_micros().min(u64::MAX as u128) as u64;
        let buf = match kind {
            RendererKind::Cpu => &self.inner.cpu,
            RendererKind::Gpu => &self.inner.gpu,
        };
        let mut g = buf.write().unwrap();
        if g.len() == SAMPLE_CAP {
            g.pop_front();
        }
        g.push_back(micros);
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            cpu: summarize(&self.inner.cpu.read().unwrap()),
            gpu: summarize(&self.inner.gpu.read().unwrap()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct LatencyStats {
    pub count: usize,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct TelemetrySnapshot {
    pub cpu: LatencyStats,
    pub gpu: LatencyStats,
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
