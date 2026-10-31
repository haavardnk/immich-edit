use rayon::prelude::*;

const BLOCK: usize = 64;
const SMOOTHING_PASSES: usize = 3;

struct Grid {
    cols: usize,
    rows: usize,
    width: usize,
    height: usize,
    cells: Vec<[f32; 3]>,
}

fn span(cell: usize, cells: usize, extent: usize) -> std::ops::Range<usize> {
    cell * extent / cells..(cell + 1) * extent / cells
}

impl Grid {
    fn differences(reference: &[f32], denoised: &[f32], width: usize, height: usize) -> Self {
        let cols = width.div_ceil(BLOCK);
        let rows = height.div_ceil(BLOCK);
        let cells = (0..rows * cols)
            .into_par_iter()
            .map(|cell| {
                let ys = span(cell / cols, rows, height);
                let xs = span(cell % cols, cols, width);
                let count = (ys.len() * xs.len()) as f32;
                let sum = ys
                    .flat_map(|y| xs.clone().map(move |x| (y * width + x) * 3))
                    .fold([0.0f32; 3], |acc, i| {
                        std::array::from_fn(|c| acc[c] + reference[i + c] - denoised[i + c])
                    });
                sum.map(|s| s / count)
            })
            .collect();
        Self {
            cols,
            rows,
            width,
            height,
            cells,
        }
    }

    fn smoothed(&self) -> Self {
        let cells = (0..self.rows * self.cols)
            .map(|cell| {
                let (gy, gx) = (cell / self.cols, cell % self.cols);
                let ys = gy.saturating_sub(1)..=(gy + 1).min(self.rows - 1);
                let xs = gx.saturating_sub(1)..=(gx + 1).min(self.cols - 1);
                let count = (ys.clone().count() * xs.clone().count()) as f32;
                let sum = ys
                    .flat_map(|y| xs.clone().map(move |x| y * self.cols + x))
                    .fold([0.0f32; 3], |acc, i| {
                        std::array::from_fn(|c| acc[c] + self.cells[i][c])
                    });
                sum.map(|s| s / count)
            })
            .collect();
        Self { cells, ..*self }
    }

    fn sample(&self, x: usize, y: usize) -> [f32; 3] {
        let (x0, x1, tx) = Self::neighbours(x, self.cols, self.width);
        let (y0, y1, ty) = Self::neighbours(y, self.rows, self.height);
        let at = |gx: usize, gy: usize| self.cells[gy * self.cols + gx];
        let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        std::array::from_fn(|ch| {
            let top = a[ch] + (b[ch] - a[ch]) * tx;
            let bottom = c[ch] + (d[ch] - c[ch]) * tx;
            top + (bottom - top) * ty
        })
    }

    fn neighbours(at: usize, cells: usize, extent: usize) -> (usize, usize, f32) {
        let pos =
            ((at as f32 + 0.5) * cells as f32 / extent as f32 - 0.5).clamp(0.0, (cells - 1) as f32);
        let low = pos.floor() as usize;
        (low, (low + 1).min(cells - 1), pos - low as f32)
    }
}

pub fn restore_low_frequencies(reference: &[f32], denoised: &mut [f32], width: usize) {
    let height = denoised.len() / (width * 3);
    let mut grid = Grid::differences(reference, denoised, width, height);
    for _ in 0..SMOOTHING_PASSES {
        grid = grid.smoothed();
    }
    denoised
        .par_chunks_exact_mut(width * 3)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                let lost = grid.sample(x, y);
                px.iter_mut().zip(lost).for_each(|(v, d)| *v += d);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: usize, height: usize) -> Vec<f32> {
        (0..width * height)
            .flat_map(|i| {
                let x = (i % width) as f32 / width as f32;
                let y = (i / width) as f32 / height as f32;
                [0.02 + 0.1 * x, 0.03 + 0.05 * y, 0.01 + 0.02 * (x + y)]
            })
            .collect()
    }

    #[test]
    fn a_uniform_shift_is_restored_exactly() {
        for (width, height) in [(128, 64), (70, 33), (129, 65), (1, 1)] {
            let reference = gradient(width, height);
            let mut shifted: Vec<f32> = reference.iter().map(|v| v - 0.004).collect();
            restore_low_frequencies(&reference, &mut shifted, width);
            let worst = shifted
                .iter()
                .zip(&reference)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(worst < 1e-6, "{width}x{height}: {worst}");
        }
    }

    #[test]
    fn a_darkened_frame_recovers_most_of_its_mean() {
        for (width, height) in [(128, 64), (70, 33), (1, 1)] {
            let reference = gradient(width, height);
            let mut denoised: Vec<f32> = reference.iter().map(|v| v * 0.9 - 0.004).collect();
            let mean_before: f32 = denoised.iter().sum::<f32>() / denoised.len() as f32;
            restore_low_frequencies(&reference, &mut denoised, width);
            let mean_after: f32 = denoised.iter().sum::<f32>() / denoised.len() as f32;
            let mean_reference: f32 = reference.iter().sum::<f32>() / reference.len() as f32;
            assert!(
                (mean_after - mean_reference).abs() < (mean_before - mean_reference).abs() / 8.0,
                "{width}x{height}: {mean_before} -> {mean_after}, want {mean_reference}"
            );
        }
    }

    #[test]
    fn removed_fine_noise_stays_removed() {
        for (width, height, row_phase, limit) in [(128, 128, 1, 1e-5), (129, 64, 0, 1e-3)] {
            let clean = gradient(width, height);
            let noisy: Vec<f32> = clean
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let px = i / 3;
                    let sign = if (px % width + px / width * row_phase) % 2 == 0 {
                        1.0
                    } else {
                        -1.0
                    };
                    v + sign * 0.02
                })
                .collect();
            let mut denoised = clean.clone();
            restore_low_frequencies(&noisy, &mut denoised, width);
            let worst = denoised
                .iter()
                .zip(&clean)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(worst < limit, "{width}x{height}: {worst}");
        }
    }
}
