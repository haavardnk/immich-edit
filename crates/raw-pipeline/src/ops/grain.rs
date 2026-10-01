use super::LinearImage;
use super::{Op, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{CropRect, Edits, EffectsEdits};
use crate::math::luma;
use rayon::prelude::*;

pub struct GrainOp;

impl Op for GrainOp {
    fn id(&self) -> &'static str {
        "grain"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Pass(super::GpuPass::Effects)
    }
    fn stage(&self) -> Stage {
        Stage::Output
    }
    fn order(&self) -> i32 {
        2
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.effects.grain_active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let e = &edits.effects;
        if !e.grain_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": e.grain_amount,
            "size": e.grain_size,
            "roughness": e.grain_roughness,
        }))
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let e: &mut EffectsEdits = &mut edits.effects;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            e.grain_amount = v;
        }
        if let Some(v) = value.get("size").and_then(|v| v.as_f64()) {
            e.grain_size = v;
        }
        if let Some(v) = value.get("roughness").and_then(|v| v.as_f64()) {
            e.grain_roughness = v;
        }
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_grain(
            image,
            &edits.effects,
            ctx.render.roi,
            ctx.render.output_scale,
        );
        Ok(())
    }
}

const GRAIN_SEED: u32 = 0x6A1A_5EED;
const FINE_SEED: u32 = GRAIN_SEED ^ 0x9E37_79B9;
const SPREAD_SALT: u32 = 0x85EB_CA6B;
const EXACT_SPAN: usize = 5;

#[derive(Clone, Copy)]
struct Window {
    start: f32,
    end: f32,
}

#[inline]
fn pcg_hash(mut x: u32) -> u32 {
    x = x.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((x >> ((x >> 28).wrapping_add(4))) ^ x).wrapping_mul(277803737);
    (word >> 22) ^ word
}

#[inline]
fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let v = pcg_hash((x as u32).wrapping_mul(0x27d4eb2d) ^ pcg_hash((y as u32) ^ seed));
    (v as f32) / (u32::MAX as f32)
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn tent(t: f32) -> f32 {
    let a = t.abs();
    if a >= 1.0 { 0.0 } else { 1.0 - fade(a) }
}

fn lattice_span(win: Window, cell: f32) -> (i32, usize) {
    let lo = (win.start.floor() / cell).floor() as i32;
    let hi = ((win.end.ceil() - 1.0) / cell).floor() as i32 + 1;
    (lo, (hi - lo + 1) as usize)
}

fn lattice_weight(i: i32, win: Window, cell: f32) -> f32 {
    let first = win.start.floor().max(((i - 1) as f32 * cell).floor()) as i32;
    let last = (win.end.ceil() - 1.0).min(((i + 1) as f32 * cell).ceil()) as i32;
    let acc: f32 = (first..=last)
        .map(|xs| {
            let cover = ((xs + 1) as f32).min(win.end) - (xs as f32).max(win.start);
            cover.max(0.0) * tent(xs as f32 / cell - i as f32)
        })
        .sum();
    acc / (win.end - win.start)
}

fn box_noise(wx: Window, wy: Window, cell: f32, seed: u32) -> f32 {
    let (x0, nx) = lattice_span(wx, cell);
    let (y0, ny) = lattice_span(wy, cell);
    if nx <= EXACT_SPAN && ny <= EXACT_SPAN {
        let ax: [f32; EXACT_SPAN] =
            std::array::from_fn(|k| lattice_weight(x0 + k as i32, wx, cell));
        let ay: [f32; EXACT_SPAN] =
            std::array::from_fn(|k| lattice_weight(y0 + k as i32, wy, cell));
        return (0..nx)
            .flat_map(|kx| (0..ny).map(move |ky| (kx, ky)))
            .map(|(kx, ky)| hash2(x0 + kx as i32, y0 + ky as i32, seed) * ax[kx] * ay[ky])
            .sum();
    }
    let ex: f32 = (0..nx)
        .map(|k| lattice_weight(x0 + k as i32, wx, cell).powi(2))
        .sum();
    let ey: f32 = (0..ny)
        .map(|k| lattice_weight(y0 + k as i32, wy, cell).powi(2))
        .sum();
    let u = hash2(
        wx.start.floor() as i32,
        wy.start.floor() as i32,
        seed ^ SPREAD_SALT,
    );
    0.5 + (u - 0.5) * (ex * ey).sqrt()
}

pub fn apply_grain(
    image: &mut LinearImage,
    e: &EffectsEdits,
    roi: Option<CropRect>,
    output_scale: f32,
) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let amount = (e.grain_amount / 100.0) as f32;
    let size = (e.grain_size / 100.0) as f32;
    let roughness = (e.grain_roughness / 100.0) as f32;
    let cell = lerp(1.0, 8.0, size);
    let fine_cell = (cell * 0.5).max(1.0);
    let r = roi.unwrap_or(CropRect::full());
    let off_x = r.x * w as f32 / r.w;
    let off_y = r.y * h as f32 / r.h;
    let step = 1.0 / output_scale;
    let strength = amount * 0.15;

    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let wy = Window {
                start: (off_y + y as f32) * step,
                end: (off_y + y as f32 + 1.0) * step,
            };
            for x in 0..w {
                let wx = Window {
                    start: (off_x + x as f32) * step,
                    end: (off_x + x as f32 + 1.0) * step,
                };
                let base = box_noise(wx, wy, cell, GRAIN_SEED);
                let fine = box_noise(wx, wy, fine_cell, FINE_SEED);
                let n = lerp(base, fine, roughness) * 2.0 - 1.0;
                let delta = n * strength;
                let i = x * 3;
                let r = row[i];
                let g = row[i + 1];
                let b = row[i + 2];
                let yv = luma(r, g, b);
                let scale = if yv > 1e-6 { (yv + delta) / yv } else { 1.0 };
                row[i] = (r * scale).clamp(0.0, 4.0);
                row[i + 1] = (g * scale).clamp(0.0, 4.0);
                row[i + 2] = (b * scale).clamp(0.0, 4.0);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};

    const Y_R: f64 = LUMA_R as f64;
    const Y_G: f64 = LUMA_G as f64;
    const Y_B: f64 = LUMA_B as f64;

    fn make_image(w: usize, h: usize, val: f32) -> LinearImage {
        LinearImage::new(vec![val; w * h * 3], w, h)
    }

    fn defaults() -> EffectsEdits {
        EffectsEdits::default()
    }

    fn mean_luma(img: &LinearImage) -> f32 {
        let mut acc = 0.0_f64;
        let n = img.width * img.height;
        for i in 0..n {
            let r = img.rgb[i * 3] as f64;
            let g = img.rgb[i * 3 + 1] as f64;
            let b = img.rgb[i * 3 + 2] as f64;
            acc += Y_R * r + Y_G * g + Y_B * b;
        }
        (acc / n as f64) as f32
    }

    fn variance(img: &LinearImage) -> f32 {
        let m = mean_luma(img) as f64;
        let mut acc = 0.0_f64;
        let n = img.width * img.height;
        for i in 0..n {
            let r = img.rgb[i * 3] as f64;
            let g = img.rgb[i * 3 + 1] as f64;
            let b = img.rgb[i * 3 + 2] as f64;
            let y = Y_R * r + Y_G * g + Y_B * b;
            acc += (y - m).powi(2);
        }
        (acc / n as f64) as f32
    }

    #[test]
    fn amount_zero_identity() {
        let mut img = make_image(32, 32, 0.5);
        let orig = img.rgb.clone();
        apply_grain(&mut img, &defaults(), None, 1.0);
        if img.rgb != orig {
            panic!("grain at 0 should be identity");
        }
    }

    #[test]
    fn variance_increases_on_flat_patch() {
        let flat = make_image(64, 64, 0.5);
        let mut grainy = make_image(64, 64, 0.5);
        let mut e = defaults();
        e.grain_amount = 80.0;
        apply_grain(&mut grainy, &e, None, 1.0);
        let v0 = variance(&flat);
        let v1 = variance(&grainy);
        if v1 <= v0 + 1e-4 {
            panic!("variance {v1} should exceed flat {v0}");
        }
    }

    #[test]
    fn deterministic_two_runs() {
        let mut a = make_image(32, 32, 0.5);
        let mut b = make_image(32, 32, 0.5);
        let mut e = defaults();
        e.grain_amount = 50.0;
        apply_grain(&mut a, &e, None, 1.0);
        apply_grain(&mut b, &e, None, 1.0);
        if a.rgb != b.rgb {
            panic!("grain should be deterministic");
        }
    }

    #[test]
    fn mean_luma_stays_within_one_percent() {
        let mut img = make_image(128, 128, 0.5);
        let before = mean_luma(&img);
        let mut e = defaults();
        e.grain_amount = 60.0;
        apply_grain(&mut img, &e, None, 1.0);
        let after = mean_luma(&img);
        let drift = (after - before).abs() / before;
        if drift > 0.01 {
            panic!("mean drifted {drift} (before {before}, after {after})");
        }
    }

    #[test]
    fn size_changes_spatial_frequency() {
        fn neighbor_corr(img: &LinearImage) -> f32 {
            let m = mean_luma(img);
            let mut num = 0.0_f64;
            let mut den = 0.0_f64;
            for y in 0..img.height {
                for x in 0..img.width - 1 {
                    let a = img.rgb[(y * img.width + x) * 3] - m;
                    let b = img.rgb[(y * img.width + x + 1) * 3] - m;
                    num += (a * b) as f64;
                    den += (a * a) as f64;
                }
            }
            if den < 1e-9 { 0.0 } else { (num / den) as f32 }
        }
        let mut small = make_image(128, 128, 0.5);
        let mut big = make_image(128, 128, 0.5);
        let mut es = defaults();
        es.grain_amount = 60.0;
        es.grain_size = 0.0;
        let mut eb = defaults();
        eb.grain_amount = 60.0;
        eb.grain_size = 100.0;
        apply_grain(&mut small, &es, None, 1.0);
        apply_grain(&mut big, &eb, None, 1.0);
        let cs = neighbor_corr(&small);
        let cb = neighbor_corr(&big);
        if cb <= cs {
            panic!("larger grain {cb} should have higher neighbor correlation than small {cs}");
        }
    }

    fn grained(side: usize, e: &EffectsEdits, output_scale: f32) -> Vec<f32> {
        let mut img = make_image(side, side, 0.5);
        apply_grain(&mut img, e, None, output_scale);
        img.rgb.chunks(3).map(|p| p[1]).collect()
    }

    fn box_downscale(full: &[f32], side: usize, k: usize) -> Vec<f32> {
        let out = side / k;
        (0..out * out)
            .map(|i| {
                let oy = i / out;
                let ox = i % out;
                let sum: f32 = (0..k * k)
                    .map(|j| full[(oy * k + j / k) * side + ox * k + j % k])
                    .sum();
                sum / (k * k) as f32
            })
            .collect()
    }

    fn std_dev(v: &[f32]) -> f32 {
        let m = v.iter().sum::<f32>() / v.len() as f32;
        (v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32).sqrt()
    }

    #[test]
    fn reduced_render_matches_downscaled_full_render() {
        let side = 256;
        for (size, roughness, k) in [(100.0, 0.0, 2), (100.0, 100.0, 4), (50.0, 50.0, 2)] {
            let e = EffectsEdits {
                grain_amount: 80.0,
                grain_size: size,
                grain_roughness: roughness,
                ..defaults()
            };
            let full = box_downscale(&grained(side, &e, 1.0), side, k);
            let reduced = grained(side / k, &e, 1.0 / k as f32);
            let worst = full
                .iter()
                .zip(&reduced)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f32, f32::max);
            if worst > 1e-4 {
                panic!("size {size} roughness {roughness} k {k}: max diff {worst}");
            }
        }
    }

    #[test]
    fn fine_grain_keeps_downscaled_strength() {
        let side = 512;
        let e = EffectsEdits {
            grain_amount: 80.0,
            grain_size: 0.0,
            grain_roughness: 0.0,
            ..defaults()
        };
        let full = grained(side, &e, 1.0);
        for k in [4, 8] {
            let want = std_dev(&box_downscale(&full, side, k));
            let got = std_dev(&grained(side / k, &e, 1.0 / k as f32));
            let ratio = got / want;
            if !(0.9..=1.1).contains(&ratio) {
                panic!("k {k}: reduced std {got} vs downscaled {want} (ratio {ratio})");
            }
        }
    }
}
