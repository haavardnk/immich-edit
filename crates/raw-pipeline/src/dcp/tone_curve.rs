#[cfg(test)]
mod tests;

const BUCKETS: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct ToneCurve {
    points: Vec<[f32; 2]>,
    starts: Vec<u32>,
}

impl ToneCurve {
    pub fn new(points: Vec<[f32; 2]>) -> Self {
        let starts = (0..=BUCKETS)
            .map(|b| match b {
                BUCKETS => points.len() as u32,
                _ => points.partition_point(|p| p[0] < b as f32 / BUCKETS as f32) as u32,
            })
            .collect();
        Self { points, starts }
    }

    pub fn points(&self) -> &[[f32; 2]] {
        &self.points
    }

    pub fn eval(&self, x: f32) -> f32 {
        let curve = self.points.as_slice();
        if curve.len() < 2 {
            return x;
        }
        let x = x.clamp(0.0, 1.0);
        if x <= curve[0][0] {
            return curve[0][1];
        }
        if x >= curve[curve.len() - 1][0] {
            return curve[curve.len() - 1][1];
        }
        let bucket = ((x * BUCKETS as f32) as usize).min(BUCKETS - 1);
        let lo = self.starts[bucket] as usize;
        let candidates = &curve[lo..self.starts[bucket + 1] as usize];
        let hi = (lo + candidates.partition_point(|p| p[0] < x)).max(1);
        let a = curve[hi - 1];
        let b = curve[hi];
        let span = b[0] - a[0];
        if span <= 1e-9 {
            return a[1];
        }
        let t = (x - a[0]) / span;
        a[1] + (b[1] - a[1]) * t
    }
}
