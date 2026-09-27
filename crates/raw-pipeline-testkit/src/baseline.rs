use serde::{Deserialize, Serialize};

const GRID: usize = 8;

#[derive(Serialize, Deserialize, PartialEq)]
pub struct RenderStats {
    pub width: u32,
    pub height: u32,
    pub mean_rgb: [f64; 3],
    pub luma_grid: Vec<f64>,
}

pub struct Tolerance {
    pub mean_rgb: f64,
    pub grid_cell: f64,
    pub grid_mean: f64,
}

impl RenderStats {
    pub fn measure(rgb: &[u8], width: usize, height: usize) -> Self {
        let n = (width * height) as f64;
        let mut sum = [0.0f64; 3];
        for px in rgb.chunks_exact(3) {
            sum[0] += px[0] as f64;
            sum[1] += px[1] as f64;
            sum[2] += px[2] as f64;
        }
        let mut grid_sum = vec![0.0f64; GRID * GRID];
        let mut grid_count = vec![0u32; GRID * GRID];
        for (i, px) in rgb.chunks_exact(3).enumerate() {
            let gx = ((i % width) * GRID / width).min(GRID - 1);
            let gy = ((i / width) * GRID / height).min(GRID - 1);
            grid_sum[gy * GRID + gx] += crate::color::luma(px);
            grid_count[gy * GRID + gx] += 1;
        }
        Self {
            width: width as u32,
            height: height as u32,
            mean_rgb: [sum[0] / n, sum[1] / n, sum[2] / n],
            luma_grid: grid_sum
                .iter()
                .zip(&grid_count)
                .map(|(s, c)| if *c == 0 { 0.0 } else { s / *c as f64 })
                .collect(),
        }
    }

    pub fn drift(&self, base: &Self, tol: &Tolerance) -> Vec<String> {
        if (self.width, self.height) != (base.width, base.height) {
            return vec![format!(
                "dims {}x{} != baseline {}x{}",
                self.width, self.height, base.width, base.height
            )];
        }
        if self.luma_grid.len() != base.luma_grid.len() {
            return vec!["grid length mismatch".to_string()];
        }
        let mut errs: Vec<String> = (0..3)
            .map(|c| (c, (self.mean_rgb[c] - base.mean_rgb[c]).abs()))
            .filter(|(_, d)| *d > tol.mean_rgb)
            .map(|(c, d)| format!("mean ch{c} diff {d:.3} > {}", tol.mean_rgb))
            .collect();
        let cells: Vec<f64> = self
            .luma_grid
            .iter()
            .zip(&base.luma_grid)
            .map(|(a, b)| (a - b).abs())
            .collect();
        let max_cell = cells.iter().copied().fold(0.0, f64::max);
        let mean_cell = cells.iter().sum::<f64>() / cells.len() as f64;
        if max_cell > tol.grid_cell {
            errs.push(format!(
                "luma grid max cell diff {max_cell:.3} > {}",
                tol.grid_cell
            ));
        }
        if mean_cell > tol.grid_mean {
            errs.push(format!(
                "luma grid mean diff {mean_cell:.3} > {}",
                tol.grid_mean
            ));
        }
        errs
    }
}
