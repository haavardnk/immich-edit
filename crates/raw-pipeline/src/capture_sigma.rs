use rayon::prelude::*;

const LOWER_LIMIT: f32 = 1000.0 / 65535.0;
const CLIP_LIMIT: f32 = 0.95;
const MAX_SIGMA: f32 = 2.0;
const CLIP_GUARD: usize = 2;

struct Cfa {
    period: usize,
    colors: Vec<u8>,
}

impl Cfa {
    fn parse(cfa_pattern: &str) -> Option<Self> {
        if let Some(pattern) = crate::cpu::demosaic::parse_xtrans(cfa_pattern) {
            return Some(Self {
                period: 6,
                colors: pattern.to_vec(),
            });
        }
        if cfa_pattern.len() == 4 && cfa_pattern.bytes().all(|b| matches!(b, b'R' | b'G' | b'B')) {
            return Some(Self {
                period: 2,
                colors: cfa_pattern.as_bytes().to_vec(),
            });
        }
        None
    }

    #[inline]
    fn is_green(&self, x: usize, y: usize) -> bool {
        self.colors[(y % self.period) * self.period + (x % self.period)] == b'G'
    }
}

pub fn estimate(data: &[f32], width: usize, height: usize, cfa_pattern: &str) -> Option<f32> {
    if data.len() != width * height || width < 16 || height < 16 {
        return None;
    }
    let cfa = Cfa::parse(cfa_pattern)?;
    let margin = CLIP_GUARD + 1;
    let max_ratio = (margin..height - margin)
        .into_par_iter()
        .map(|y| row_max_ratio(data, width, &cfa, y, margin))
        .reduce(|| 1.0f32, f32::max);
    let curvature = max_ratio.ln();
    if curvature <= 0.0 {
        return None;
    }
    let sigma = (1.0 / curvature).sqrt();
    if !sigma.is_finite() || sigma <= 0.0 {
        return None;
    }
    Some(sigma.min(MAX_SIGMA))
}

fn row_max_ratio(data: &[f32], width: usize, cfa: &Cfa, y: usize, margin: usize) -> f32 {
    let mut max_ratio = 1.0f32;
    for x in margin..width - margin {
        if !cfa.is_green(x, y) {
            continue;
        }
        let center = data[y * width + x];
        if center <= 0.0 {
            continue;
        }
        for dx in [-1i32, 1] {
            let nx = (x as i32 + dx) as usize;
            let ny = y + 1;
            if !cfa.is_green(nx, ny) {
                continue;
            }
            let neighbor = data[ny * width + nx];
            if neighbor <= 0.0 {
                continue;
            }
            let max_val = center.max(neighbor);
            if max_val <= LOWER_LIMIT {
                continue;
            }
            let min_val = center.min(neighbor);
            if max_val <= max_ratio * min_val {
                continue;
            }
            if clipped_near(data, width, x, y) || clipped_near(data, width, nx, ny) {
                continue;
            }
            max_ratio = max_val / min_val;
        }
    }
    max_ratio
}

fn clipped_near(data: &[f32], width: usize, x: usize, y: usize) -> bool {
    (y - CLIP_GUARD..=y + CLIP_GUARD)
        .flat_map(|yy| (x - CLIP_GUARD..=x + CLIP_GUARD).map(move |xx| (yy, xx)))
        .any(|(yy, xx)| data[yy * width + xx] >= CLIP_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAYER: &str = "RGGB";
    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

    fn blurred_edges(width: usize, height: usize, sigma: f32, dark: f32, bright: f32) -> Vec<f32> {
        let radius = (3.0 * sigma).ceil() as i32;
        let mut kernel: Vec<f32> = (-radius..=radius)
            .map(|i| (-((i * i) as f32) / (2.0 * sigma * sigma)).exp())
            .collect();
        let sum: f32 = kernel.iter().sum();
        for k in kernel.iter_mut() {
            *k /= sum;
        }
        let step = |x: usize, y: usize| -> f32 {
            if (x / 17 + y / 23) % 2 == 0 {
                bright
            } else {
                dark
            }
        };
        let mut tmp = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                tmp[y * width + x] = kernel
                    .iter()
                    .enumerate()
                    .map(|(i, k)| {
                        let sx = (x as i32 + i as i32 - radius).clamp(0, width as i32 - 1) as usize;
                        k * step(sx, y)
                    })
                    .sum();
            }
        }
        let mut data = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                data[y * width + x] = kernel
                    .iter()
                    .enumerate()
                    .map(|(i, k)| {
                        let sy =
                            (y as i32 + i as i32 - radius).clamp(0, height as i32 - 1) as usize;
                        k * tmp[sy * width + x]
                    })
                    .sum();
            }
        }
        data
    }

    #[test]
    fn estimate_grows_with_blur() {
        let mut previous = 0.0f32;
        for target in [0.5f32, 0.8, 1.2, 1.8] {
            let data = blurred_edges(128, 128, target, 0.05, 0.9);
            let sigma = estimate(&data, 128, 128, BAYER).expect("sigma");
            eprintln!("blur {target} -> sigma {sigma}");
            assert!(
                sigma > previous,
                "sigma {sigma} did not grow past {previous} at blur {target}"
            );
            assert!(
                sigma <= MAX_SIGMA,
                "sigma {sigma} exceeded cap at blur {target}"
            );
            previous = sigma;
        }
    }

    #[test]
    fn flat_field_has_no_estimate() {
        let data = vec![0.4f32; 64 * 64];
        assert_eq!(estimate(&data, 64, 64, BAYER), None);
    }

    #[test]
    fn near_flat_field_saturates_to_cap() {
        let mut data = vec![0.4f32; 64 * 64];
        data[33 * 64 + 32] = 0.4001;
        assert_eq!(estimate(&data, 64, 64, BAYER), Some(MAX_SIGMA));
    }

    #[test]
    fn clipped_highlights_do_not_move_the_estimate() {
        let mut data = blurred_edges(128, 128, 1.2, 0.05, 0.9);
        let clean = estimate(&data, 128, 128, BAYER).expect("sigma");
        for y in 40..48 {
            for x in 40..48 {
                data[y * 128 + x] = 1.0;
            }
        }
        let clipped = estimate(&data, 128, 128, BAYER).expect("sigma");
        eprintln!("clean {clean} clipped {clipped}");
        assert!(
            (clipped - clean).abs() < 0.05,
            "clipped block moved the estimate from {clean} to {clipped}"
        );
    }

    #[test]
    fn xtrans_tracks_bayer() {
        let data = blurred_edges(192, 192, 1.0, 0.05, 0.9);
        let bayer = estimate(&data, 192, 192, BAYER).expect("bayer sigma");
        let xtrans = estimate(&data, 192, 192, XTRANS).expect("xtrans sigma");
        eprintln!("bayer {bayer} xtrans {xtrans}");
        assert!(
            (bayer - xtrans).abs() < 0.05,
            "bayer {bayer} and xtrans {xtrans} disagree"
        );
    }

    #[test]
    fn rejects_unknown_pattern() {
        let data = vec![0.4f32; 64 * 64];
        assert_eq!(estimate(&data, 64, 64, ""), None);
        assert_eq!(estimate(&data, 64, 64, "RGB"), None);
    }
}
