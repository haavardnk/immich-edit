use super::LinearImage;
use rayon::prelude::*;

pub fn luma_buffer(image: &LinearImage) -> Vec<f32> {
    image
        .rgb
        .par_chunks_exact(3)
        .map(|px| 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2])
        .collect()
}

pub fn box_blur_separable(src: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    if radius == 0 {
        return src.to_vec();
    }
    let horiz: Vec<f32> = (0..height)
        .into_par_iter()
        .flat_map(|y| {
            let row_start = y * width;
            let mut out = vec![0.0f32; width];
            for (x, slot) in out.iter_mut().enumerate() {
                let x0 = x.saturating_sub(radius);
                let x1 = (x + radius).min(width - 1);
                let mut sum = 0.0f32;
                for xi in x0..=x1 {
                    sum += src[row_start + xi];
                }
                *slot = sum / (x1 - x0 + 1) as f32;
            }
            out
        })
        .collect();
    let mut dst = vec![0.0f32; width * height];
    dst.par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let y0 = y.saturating_sub(radius);
            let y1 = (y + radius).min(height - 1);
            let denom = (y1 - y0 + 1) as f32;
            for (x, slot) in row.iter_mut().enumerate() {
                let mut sum = 0.0f32;
                for yi in y0..=y1 {
                    sum += horiz[yi * width + x];
                }
                *slot = sum / denom;
            }
        });
    dst
}

pub fn apply_luma_delta(image: &mut LinearImage, new_luma: &[f32]) {
    image
        .rgb
        .par_chunks_exact_mut(3)
        .zip(new_luma.par_iter())
        .for_each(|(px, &target)| {
            let r = px[0];
            let g = px[1];
            let b = px[2];
            let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            if l <= 1e-5 {
                px[0] = target;
                px[1] = target;
                px[2] = target;
                return;
            }
            let scale = target / l;
            px[0] = (r * scale).max(0.0);
            px[1] = (g * scale).max(0.0);
            px[2] = (b * scale).max(0.0);
        });
}
