#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeWindow {
    pub min: f32,
    pub max: f32,
    pub softness: f32,
}

impl RangeWindow {
    pub fn is_full(&self) -> bool {
        self.min <= 0.0 && self.max >= 1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BakeParams {
    pub grow: f32,
    pub feather: f32,
    pub guided_radius: usize,
    pub guided_eps: f32,
    pub range: Option<RangeWindow>,
}

impl Default for BakeParams {
    fn default() -> Self {
        Self {
            grow: 0.0,
            feather: 0.0,
            guided_radius: 8,
            guided_eps: 1e-4,
            range: None,
        }
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge1 - edge0 <= 1e-6 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn range_weight(value: f32, window: RangeWindow) -> f32 {
    let lo = window.min.min(window.max);
    let hi = window.min.max(window.max);
    let soft = window.softness.max(0.0);
    if soft <= 1e-6 {
        return if value >= lo && value <= hi { 1.0 } else { 0.0 };
    }
    let rising = smoothstep(lo - soft, lo, value);
    let falling = 1.0 - smoothstep(hi, hi + soft, value);
    rising.min(falling)
}

fn integral(src: &[f32], w: usize, h: usize) -> Vec<f64> {
    let mut sat = vec![0.0f64; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0.0f64;
        for x in 0..w {
            row += src[y * w + x] as f64;
            sat[(y + 1) * (w + 1) + x + 1] = sat[y * (w + 1) + x + 1] + row;
        }
    }
    sat
}

pub fn box_blur(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let sat = integral(src, w, h);
    let mut out = vec![0.0f32; w * h];
    let r = radius as isize;
    for y in 0..h {
        let y0 = (y as isize - r).max(0) as usize;
        let y1 = ((y as isize + r + 1) as usize).min(h);
        for x in 0..w {
            let x0 = (x as isize - r).max(0) as usize;
            let x1 = ((x as isize + r + 1) as usize).min(w);
            let sum = sat[y1 * (w + 1) + x1] - sat[y0 * (w + 1) + x1] - sat[y1 * (w + 1) + x0]
                + sat[y0 * (w + 1) + x0];
            let area = ((x1 - x0) * (y1 - y0)) as f64;
            out[y * w + x] = (sum / area) as f32;
        }
    }
    out
}

pub fn guided_filter(
    guide: &[f32],
    src: &[f32],
    w: usize,
    h: usize,
    radius: usize,
    eps: f32,
) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let mean_i = box_blur(guide, w, h, radius);
    let mean_p = box_blur(src, w, h, radius);
    let ii: Vec<f32> = guide.iter().map(|v| v * v).collect();
    let ip: Vec<f32> = guide.iter().zip(src).map(|(g, p)| g * p).collect();
    let corr_i = box_blur(&ii, w, h, radius);
    let corr_ip = box_blur(&ip, w, h, radius);

    let mut a = vec![0.0f32; w * h];
    let mut b = vec![0.0f32; w * h];
    for i in 0..w * h {
        let var = corr_i[i] - mean_i[i] * mean_i[i];
        let cov = corr_ip[i] - mean_i[i] * mean_p[i];
        a[i] = cov / (var + eps);
        b[i] = mean_p[i] - a[i] * mean_i[i];
    }
    let mean_a = box_blur(&a, w, h, radius);
    let mean_b = box_blur(&b, w, h, radius);
    (0..w * h)
        .map(|i| (mean_a[i] * guide[i] + mean_b[i]).clamp(0.0, 1.0))
        .collect()
}

fn edt_1d(f: &[f32], out: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32))
                / (2.0 * q as f32 - 2.0 * p as f32);
            if s <= z[k] {
                if k == 0 {
                    v[0] = q;
                    z[0] = f32::NEG_INFINITY;
                    z[1] = f32::INFINITY;
                    break;
                }
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
        }
    }
    k = 0;
    for (q, slot) in out.iter_mut().enumerate().take(n) {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let p = v[k];
        let d = q as f32 - p as f32;
        *slot = d * d + f[p];
    }
}

fn distance_transform(binary: &[bool], w: usize, h: usize) -> Vec<f32> {
    let big = (w * w + h * h) as f32;
    let mut grid: Vec<f32> = binary.iter().map(|b| if *b { 0.0 } else { big }).collect();
    let n = w.max(h);
    let mut f = vec![0.0f32; n];
    let mut out = vec![0.0f32; n];
    let mut v = vec![0usize; n];
    let mut z = vec![0.0f32; n + 1];

    for x in 0..w {
        for y in 0..h {
            f[y] = grid[y * w + x];
        }
        edt_1d(&f[..h], &mut out[..h], &mut v[..h], &mut z[..h + 1]);
        for y in 0..h {
            grid[y * w + x] = out[y];
        }
    }
    for y in 0..h {
        f[..w].copy_from_slice(&grid[y * w..y * w + w]);
        edt_1d(&f[..w], &mut out[..w], &mut v[..w], &mut z[..w + 1]);
        for x in 0..w {
            grid[y * w + x] = out[x].sqrt();
        }
    }
    grid
}

pub fn grow(mask: &[f32], w: usize, h: usize, pixels: f32) -> Vec<f32> {
    if pixels.abs() < 0.5 || w == 0 || h == 0 {
        return mask.to_vec();
    }
    let inside: Vec<bool> = mask.iter().map(|v| *v >= 0.5).collect();
    let outside: Vec<bool> = inside.iter().map(|v| !*v).collect();
    let d_out = distance_transform(&inside, w, h);
    let d_in = distance_transform(&outside, w, h);
    (0..w * h)
        .map(|i| {
            let signed = if inside[i] { -d_in[i] } else { d_out[i] };
            let shifted = signed - pixels;
            (0.5 - shifted).clamp(0.0, 1.0)
        })
        .collect()
}

pub fn bake(prob: &[f32], guide: &[f32], w: usize, h: usize, params: BakeParams) -> Vec<u8> {
    let windowed = match params.range {
        Some(window) => prob.iter().map(|v| range_weight(*v, window)).collect(),
        None => prob.to_vec(),
    };
    let mut mask = if params.guided_radius > 0 {
        guided_filter(
            guide,
            &windowed,
            w,
            h,
            params.guided_radius,
            params.guided_eps,
        )
    } else {
        windowed
    };
    if params.grow.abs() >= 0.5 {
        mask = grow(&mask, w, h, params.grow);
    }
    if params.feather >= 1.0 {
        let r = params.feather.round() as usize;
        mask = box_blur(&mask, w, h, r);
        mask = box_blur(&mask, w, h, r);
    }
    mask.iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_mask(w: usize, h: usize) -> Vec<f32> {
        (0..w * h)
            .map(|i| if (i % w) < w / 2 { 1.0 } else { 0.0 })
            .collect()
    }

    #[test]
    fn box_blur_preserves_constant() {
        let src = vec![0.5f32; 64];
        let out = box_blur(&src, 8, 8, 2);
        assert!(out.iter().all(|v| (v - 0.5).abs() < 1e-5));
    }

    #[test]
    fn box_blur_zero_radius_is_identity() {
        let src = step_mask(8, 8);
        assert_eq!(box_blur(&src, 8, 8, 0), src);
    }

    #[test]
    fn guided_filter_snaps_to_guide_edge() {
        let w = 32;
        let h = 8;
        let guide = step_mask(w, h);
        let blurred = box_blur(&guide, w, h, 4);
        let out = guided_filter(&guide, &blurred, w, h, 4, 1e-6);
        let left = out[h / 2 * w + 2];
        let right = out[h / 2 * w + w - 3];
        assert!(left > 0.9, "left {left}");
        assert!(right < 0.1, "right {right}");
    }

    #[test]
    fn distance_transform_is_zero_inside() {
        let w = 8;
        let h = 8;
        let binary: Vec<bool> = (0..w * h).map(|i| i % w == 0).collect();
        let d = distance_transform(&binary, w, h);
        assert!(d[0].abs() < 1e-5);
        assert!((d[3] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn grow_expands_and_shrinks() {
        let w = 32;
        let h = 8;
        let base = step_mask(w, h);
        let bigger = grow(&base, w, h, 3.0);
        let smaller = grow(&base, w, h, -3.0);
        let row = h / 2 * w;
        let covered = |m: &Vec<f32>| m[row..row + w].iter().filter(|v| **v >= 0.5).count();
        assert!(covered(&bigger) > covered(&base));
        assert!(covered(&smaller) < covered(&base));
    }

    #[test]
    fn bake_returns_full_range_bytes() {
        let w = 32;
        let h = 8;
        let prob = step_mask(w, h);
        let guide = step_mask(w, h);
        let out = bake(&prob, &guide, w, h, BakeParams::default());
        assert_eq!(out.len(), w * h);
        assert_eq!(out[0], 255);
        assert_eq!(out[w - 1], 0);
    }

    #[test]
    fn range_weight_selects_a_band_with_soft_edges() {
        let window = RangeWindow {
            min: 0.4,
            max: 0.6,
            softness: 0.1,
        };
        assert_eq!(range_weight(0.5, window), 1.0);
        assert_eq!(range_weight(0.2, window), 0.0);
        assert_eq!(range_weight(0.9, window), 0.0);
        let edge = range_weight(0.35, window);
        assert!(edge > 0.0 && edge < 1.0);
    }

    #[test]
    fn bake_keeps_only_the_selected_depth_band() {
        let w = 8;
        let h = 1;
        let prob: Vec<f32> = (0..w).map(|x| x as f32 / (w - 1) as f32).collect();
        let guide = vec![0.0f32; w];
        let params = BakeParams {
            guided_radius: 0,
            range: Some(RangeWindow {
                min: 0.0,
                max: 0.3,
                softness: 0.0,
            }),
            ..Default::default()
        };
        let out = bake(&prob, &guide, w, h, params);
        assert_eq!(out[0], 255);
        assert_eq!(out[w - 1], 0);
    }
}
