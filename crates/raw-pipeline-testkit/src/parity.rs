use raw_pipeline::frame::RenderedImage;

fn report_mode() -> bool {
    std::env::var("PARITY_REPORT").is_ok_and(|v| v == "1")
}

pub fn require_same_dims(label: &str, cpu: &RenderedImage, gpu: &RenderedImage) {
    if cpu.width != gpu.width || cpu.height != gpu.height {
        panic!(
            "{label}: dim mismatch CPU {}x{} vs GPU {}x{}",
            cpu.width, cpu.height, gpu.width, gpu.height
        );
    }
}

pub struct ParityLedger {
    what: &'static str,
    rows: Vec<(String, f64, f64)>,
}

impl ParityLedger {
    pub fn new(what: &'static str) -> Self {
        Self {
            what,
            rows: Vec::new(),
        }
    }

    pub fn check(&mut self, label: &str, cpu: &[u8], gpu: &[u8], limit: f64) -> f64 {
        let delta = mean_abs_delta(cpu, gpu);
        eprintln!(
            "PARITY {}/{label} delta={delta:.4} limit={limit}",
            self.what
        );
        self.rows.push((label.to_string(), delta, limit));
        delta
    }

    pub fn finish(self) {
        let failed: Vec<String> = self
            .rows
            .iter()
            .filter(|(_, delta, limit)| delta > limit)
            .map(|(label, delta, limit)| format!("{label}: {delta:.4} > {limit}"))
            .collect();
        if failed.is_empty() {
            return;
        }
        if report_mode() {
            eprintln!("PARITY {} over limit: {}", self.what, failed.join("; "));
            return;
        }
        panic!("{} CPU/GPU drift: {}", self.what, failed.join("; "));
    }
}

pub fn mean_abs_delta(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() {
        panic!("len mismatch: {} vs {}", a.len(), b.len());
    }
    let sum: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as i32 - y as i32).unsigned_abs() as u64)
        .sum();
    sum as f64 / a.len() as f64
}
