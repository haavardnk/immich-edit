use super::LinearImage;
use super::lens_ca::ca_scales;
use super::lens_distortion::{distortion_coeffs, distortion_zoom};
use super::lens_vignette::{vignette_coeffs, vignette_correction};
use super::sample::{sample_channel_bicubic, sample_rgb_bicubic};
use crate::edits::LensEdits;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LensCorrection {
    pub k: [f32; 3],
    pub zoom: f32,
    pub vignette: [f32; 4],
    pub ca: [f32; 2],
}

impl Default for LensCorrection {
    fn default() -> Self {
        Self {
            k: [0.0; 3],
            zoom: 1.0,
            vignette: [0.0; 4],
            ca: [1.0; 2],
        }
    }
}

impl LensCorrection {
    pub fn from_edits(lens: &LensEdits) -> Self {
        let (k1, k2, k3) = distortion_coeffs(lens);
        let (vk1, vk2, vk3, amount) = vignette_coeffs(lens);
        let (red, blue) = ca_scales(lens);
        Self {
            k: [k1, k2, k3],
            zoom: distortion_zoom(lens),
            vignette: [vk1, vk2, vk3, amount],
            ca: [red, blue],
        }
    }

    fn warps(&self) -> bool {
        self.k != [0.0; 3] || self.zoom != 1.0 || self.ca != [1.0; 2]
    }

    fn vignettes(&self) -> bool {
        self.vignette[3] != 0.0 && self.vignette[..3] != [0.0; 3]
    }
}

struct Geometry {
    lens: LensCorrection,
    cx: f32,
    cy: f32,
    inv_diag: f32,
}

impl Geometry {
    #[inline(always)]
    fn gain(&self, r: f32) -> f32 {
        let [vk1, vk2, vk3, amount] = self.lens.vignette;
        vignette_correction(vk1, vk2, vk3, amount, r)
    }

    #[inline(always)]
    fn source(&self, dx: f32, dy: f32) -> (f32, f32, f32) {
        let [k1, k2, k3] = self.lens.k;
        let r = (dx * dx + dy * dy).sqrt() * self.inv_diag;
        let r2 = r * r;
        let r4 = r2 * r2;
        let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r4 * r2;
        (dx * s + self.cx - 0.5, dy * s + self.cy - 0.5, r)
    }
}

pub fn apply_lens_correction(image: &mut LinearImage, lens: &LensCorrection) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 || !(lens.warps() || lens.vignettes()) {
        return;
    }
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let geo = Geometry {
        lens: *lens,
        cx,
        cy,
        inv_diag: 1.0 / (0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt()),
    };
    if !lens.warps() {
        image
            .rgb
            .par_chunks_mut(w * 3)
            .enumerate()
            .for_each(|(y, row)| {
                let dy = y as f32 + 0.5 - cy;
                for (x, px) in row.chunks_exact_mut(3).enumerate() {
                    let dx = x as f32 + 0.5 - cx;
                    let gain = geo.gain((dx * dx + dy * dy).sqrt() * geo.inv_diag);
                    for v in px {
                        *v = (*v * gain).max(0.0);
                    }
                }
            });
        return;
    }
    let zoom = lens.zoom;
    let src = &image.rgb;
    let [red, blue] = lens.ca;
    let mut out = vec![0.0f32; src.len()];
    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        let dy = (y as f32 + 0.5 - cy) * zoom;
        for (x, px) in row.chunks_exact_mut(3).enumerate() {
            let dx = (x as f32 + 0.5 - cx) * zoom;
            let (gx, gy, gr) = geo.source(dx, dy);
            let g_gain = geo.gain(gr);
            if red == 1.0 && blue == 1.0 {
                let sample = sample_rgb_bicubic(src, w, h, gx, gy);
                for (v, s) in px.iter_mut().zip(sample) {
                    *v = (s * g_gain).max(0.0);
                }
                continue;
            }
            let (rx, ry, rr) = geo.source(dx * red, dy * red);
            let (bx, by, br) = geo.source(dx * blue, dy * blue);
            px[0] = (sample_channel_bicubic(src, w, h, rx, ry, 0) * geo.gain(rr)).max(0.0);
            px[1] = (sample_channel_bicubic(src, w, h, gx, gy, 1) * g_gain).max(0.0);
            px[2] = (sample_channel_bicubic(src, w, h, bx, by, 2) * geo.gain(br)).max(0.0);
        }
    });
    image.rgb = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(w: usize, h: usize) -> LinearImage {
        let rgb = (0..w * h * 3)
            .map(|i| {
                let p = i / 3;
                let (x, y) = ((p % w) as f32, (p / w) as f32);
                0.1 + 0.5 * x / w as f32 + 0.3 * y / h as f32 + 0.05 * (i % 3) as f32
            })
            .collect();
        LinearImage::new(rgb, w, h)
    }

    #[test]
    fn default_is_identity() {
        let mut img = ramp(40, 30);
        let before = img.rgb.clone();
        apply_lens_correction(&mut img, &LensCorrection::default());
        if img.rgb != before {
            panic!("identity correction changed pixels");
        }
    }

    #[test]
    fn fused_pass_matches_sequential_components() {
        let (w, h) = (96usize, 64usize);
        let full = LensCorrection {
            k: [-0.08, 0.02, 0.0],
            zoom: 1.0,
            vignette: [-0.3, 0.0, 0.0, 0.8],
            ca: [1.004, 0.996],
        };
        let mut fused = ramp(w, h);
        apply_lens_correction(&mut fused, &full);
        let mut staged = ramp(w, h);
        for part in [
            LensCorrection {
                k: full.k,
                ..Default::default()
            },
            LensCorrection {
                vignette: full.vignette,
                ..Default::default()
            },
            LensCorrection {
                ca: full.ca,
                ..Default::default()
            },
        ] {
            apply_lens_correction(&mut staged, &part);
        }
        let interior = |i: &usize| {
            let p = i / 3;
            let (x, y) = (p % w, p / w);
            (8..w - 8).contains(&x) && (8..h - 8).contains(&y)
        };
        let worst = (0..w * h * 3)
            .filter(interior)
            .map(|i| (fused.rgb[i] - staged.rgb[i]).abs())
            .fold(0.0f32, f32::max);
        if worst > 2e-3 {
            panic!("fused lens pass drifted {worst} from the staged passes");
        }
    }
}
