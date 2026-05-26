use super::LinearImage;
use super::sample::sample_rgb_bicubic;
use super::{OpContext, OpMeta, SpatialOp, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};
use rayon::prelude::*;

pub struct LensDistortionOp;

impl OpMeta for LensDistortionOp {
    fn id(&self) -> &'static str {
        "lens_distortion"
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        0
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.distortion_active()
    }
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
}

impl SpatialOp for LensDistortionOp {
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_lens_distortion(image, &edits.lens);
        Ok(())
    }
}

pub fn distortion_coeffs(lens: &LensEdits) -> (f32, f32, f32) {
    if !lens.profile_enabled {
        return (0.0, 0.0, 0.0);
    }
    let (k1, k2, k3) = lens.effective_k();
    (k1 as f32, k2 as f32, k3 as f32)
}

pub fn constrain_zoom(k1: f32, k2: f32, k3: f32) -> f32 {
    let s = |r: f32| {
        let r2 = r * r;
        1.0 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2
    };
    if s(1.0) <= 1.0 {
        return 1.0;
    }
    let mut z: f32 = 1.0;
    for _ in 0..32 {
        let sz = s(z);
        if sz <= 1.0 {
            return 1.0;
        }
        let next = 1.0 / sz;
        if (next - z).abs() < 1e-6 {
            return next;
        }
        z = next;
    }
    z
}

pub fn distortion_zoom(lens: &LensEdits) -> f32 {
    if !lens.profile_enabled || !lens.constrain_crop {
        return 1.0;
    }
    let (k1, k2, k3) = distortion_coeffs(lens);
    constrain_zoom(k1, k2, k3)
}

#[allow(clippy::too_many_arguments)]
pub fn output_px_to_source_px(
    k1: f32,
    k2: f32,
    k3: f32,
    zoom: f32,
    width: u32,
    height: u32,
    dst_x: f32,
    dst_y: f32,
) -> (f32, f32) {
    let w = width as f32;
    let h = height as f32;
    if w == 0.0 || h == 0.0 {
        return (dst_x, dst_y);
    }
    let cx = w * 0.5;
    let cy = h * 0.5;
    let r_norm = 0.5 * (w * w + h * h).sqrt();
    let inv_norm = zoom / r_norm;
    let dx = (dst_x + 0.5 - cx) * inv_norm;
    let dy = (dst_y + 0.5 - cy) * inv_norm;
    let r2 = dx * dx + dy * dy;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
    let sx = dx * s * r_norm + cx - 0.5;
    let sy = dy * s * r_norm + cy - 0.5;
    (sx, sy)
}

#[derive(Debug, Clone, Copy)]
pub struct LensWarpParams {
    pub k1: f32,
    pub k2: f32,
    pub k3: f32,
    pub zoom: f32,
    pub width: u32,
    pub height: u32,
}

impl LensWarpParams {
    pub fn from_edits(lens: &LensEdits, width: u32, height: u32) -> Self {
        let (k1, k2, k3) = distortion_coeffs(lens);
        let zoom = distortion_zoom(lens);
        Self {
            k1,
            k2,
            k3,
            zoom,
            width,
            height,
        }
    }
    pub fn is_identity(&self) -> bool {
        self.k1 == 0.0 && self.k2 == 0.0 && self.k3 == 0.0 && self.zoom == 1.0
    }
    fn scale(&self, r: f32) -> f32 {
        let r2 = r * r;
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        1.0 + self.k1 * r2 + self.k2 * r4 + self.k3 * r6
    }
}

pub fn mask_uv_to_scene_uv(p: &LensWarpParams, uv: [f32; 2]) -> [f32; 2] {
    if p.is_identity() {
        return uv;
    }
    let w = p.width as f32;
    let h = p.height as f32;
    if w == 0.0 || h == 0.0 {
        return uv;
    }
    let half_diag = 0.5 * (w * w + h * h).sqrt();
    let nx = (uv[0] - 0.5) * w;
    let ny = (uv[1] - 0.5) * h;
    let r = (nx * nx + ny * ny).sqrt() * p.zoom / half_diag;
    let s = p.scale(r);
    [
        0.5 + (uv[0] - 0.5) * p.zoom * s,
        0.5 + (uv[1] - 0.5) * p.zoom * s,
    ]
}

pub fn scene_uv_to_mask_uv(p: &LensWarpParams, uv: [f32; 2]) -> [f32; 2] {
    if p.is_identity() {
        return uv;
    }
    let w = p.width as f32;
    let h = p.height as f32;
    if w == 0.0 || h == 0.0 {
        return uv;
    }
    let half_diag = 0.5 * (w * w + h * h).sqrt();
    let sx = (uv[0] - 0.5) * w;
    let sy = (uv[1] - 0.5) * h;
    let len_scene = (sx * sx + sy * sy).sqrt();
    if len_scene < 1e-9 {
        return [0.5, 0.5];
    }
    let target = len_scene / half_diag;
    let mut lo = 0.0f32;
    let mut hi = 2.0f32;
    for _ in 0..48 {
        let mid = 0.5 * (lo + hi);
        let r = mid * p.zoom;
        if mid * p.zoom * p.scale(r) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let u = 0.5 * (lo + hi);
    let mask_len = u * half_diag;
    let mx = sx / len_scene * mask_len;
    let my = sy / len_scene * mask_len;
    [0.5 + mx / w, 0.5 + my / h]
}

pub fn apply_lens_distortion(image: &mut LinearImage, lens: &LensEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let (k1, k2, k3) = distortion_coeffs(lens);
    let zoom = distortion_zoom(lens);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let r_norm = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
    let inv_norm = zoom / r_norm;
    let src = image.rgb.clone();
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let dy = (y as f32 + 0.5 - cy) * inv_norm;
            for x in 0..w {
                let dx = (x as f32 + 0.5 - cx) * inv_norm;
                let r2 = dx * dx + dy * dy;
                let r4 = r2 * r2;
                let r6 = r4 * r2;
                let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
                let sx = dx * s * r_norm + cx - 0.5;
                let sy = dy * s * r_norm + cy - 0.5;
                let sample = sample_rgb_bicubic(&src, w, h, sx, sy);
                let i = x * 3;
                row[i] = sample[0];
                row[i + 1] = sample[1];
                row[i + 2] = sample[2];
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;
    use crate::ops::{OpScratch, RenderContext};

    #[test]
    fn constrain_zoom_solves_fixed_point() {
        let k1 = 0.18;
        let z = constrain_zoom(k1, 0.0, 0.0);
        let s = 1.0 + k1 * z * z;
        if (z * s - 1.0).abs() > 1e-4 {
            panic!("z*s(z) should equal 1, got {}", z * s);
        }
        if z >= 1.0 {
            panic!("barrel should require zoom < 1, got {z}");
        }
    }

    #[test]
    fn constrain_zoom_noop_for_pincushion() {
        let z = constrain_zoom(-0.1, 0.0, 0.0);
        if (z - 1.0).abs() > 1e-6 {
            panic!("pincushion should not crop, got {z}");
        }
    }

    fn gradient_image(w: usize, h: usize) -> LinearImage {
        let mut rgb = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 3;
                let v = (x as f32) / (w as f32 - 1.0);
                rgb[i] = v;
                rgb[i + 1] = v;
                rgb[i + 2] = v;
            }
        }
        LinearImage::new(rgb, w, h)
    }

    fn ctx() -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: crate::color::identity_3x3(),
                is_raw: false,
                preview_mode: PreviewMode::None,
            },
            scratch: OpScratch { shadows_blur: None },
        }
    }

    #[test]
    fn inactive_is_identity() {
        let mut img = gradient_image(32, 24);
        let before = img.rgb.clone();
        let edits = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 50.0,
                ..Default::default()
            },
            ..Default::default()
        };
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &edits)
            .unwrap();
        for (a, b) in img.rgb.iter().zip(before.iter()) {
            if (a - b).abs() > 1e-4 {
                panic!("inactive (k=0) should not modify image; got {a} vs {b}");
            }
        }
    }

    #[test]
    fn barrel_then_pincushion_round_trip_close() {
        let mut img = gradient_image(64, 48);
        let target = img.rgb.clone();
        let barrel = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 100.0,
                k1: -0.1,
                ..Default::default()
            },
            ..Default::default()
        };
        let pincushion = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 100.0,
                k1: 0.1,
                ..Default::default()
            },
            ..Default::default()
        };
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &barrel)
            .unwrap();
        if img.rgb == target {
            panic!("barrel should modify image");
        }
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &pincushion)
            .unwrap();
        let center_idx = (24 * 64 + 32) * 3;
        if (img.rgb[center_idx] - target[center_idx]).abs() > 0.05 {
            panic!(
                "round trip too far at center: got {}, expected {}",
                img.rgb[center_idx], target[center_idx]
            );
        }
    }

    fn warp(k1: f32, zoom: f32) -> LensWarpParams {
        LensWarpParams {
            k1,
            k2: 0.0,
            k3: 0.0,
            zoom,
            width: 6000,
            height: 4000,
        }
    }

    #[test]
    fn warp_identity_is_passthrough() {
        let p = LensWarpParams {
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            zoom: 1.0,
            width: 100,
            height: 100,
        };
        let uv = [0.31, 0.78];
        let out = mask_uv_to_scene_uv(&p, uv);
        if (out[0] - uv[0]).abs() > 1e-7 || (out[1] - uv[1]).abs() > 1e-7 {
            panic!("identity mismatch: {out:?}");
        }
    }

    #[test]
    fn warp_round_trip() {
        for (k1, zoom) in [(0.15f32, 1.0f32), (-0.12, 1.0), (0.2, 0.85)] {
            let p = warp(k1, zoom);
            for uv in [[0.1, 0.2], [0.5, 0.5], [0.7, 0.9], [0.95, 0.05]] {
                let scene = mask_uv_to_scene_uv(&p, uv);
                let back = scene_uv_to_mask_uv(&p, scene);
                let dx = back[0] - uv[0];
                let dy = back[1] - uv[1];
                if dx.abs() > 1e-4 || dy.abs() > 1e-4 {
                    panic!("round trip failed k1={k1} zoom={zoom} uv={uv:?} back={back:?}");
                }
            }
        }
    }

    #[test]
    fn warp_matches_cpu_pixel_formula() {
        let lens = LensEdits {
            profile_enabled: true,
            distortion_amount: 100.0,
            k1: 0.18,
            ..Default::default()
        };
        let w: u32 = 200;
        let h: u32 = 150;
        let p = LensWarpParams::from_edits(&lens, w, h);
        let (k1, k2, k3) = distortion_coeffs(&lens);
        let zoom = distortion_zoom(&lens);
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let r_norm = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
        let inv_norm = zoom / r_norm;
        for &(x, y) in &[(0u32, 0u32), (50, 30), (100, 75), (199, 149)] {
            let dx = (x as f32 + 0.5 - cx) * inv_norm;
            let dy = (y as f32 + 0.5 - cy) * inv_norm;
            let r2 = dx * dx + dy * dy;
            let s = 1.0 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
            let sx_px = dx * s * r_norm + cx;
            let sy_px = dy * s * r_norm + cy;
            let mask_uv = [(x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32];
            let scene_uv = mask_uv_to_scene_uv(&p, mask_uv);
            let got_sx = scene_uv[0] * w as f32;
            let got_sy = scene_uv[1] * h as f32;
            if (got_sx - sx_px).abs() > 1e-3 || (got_sy - sy_px).abs() > 1e-3 {
                panic!("mismatch at ({x},{y}): cpu=({sx_px},{sy_px}) helper=({got_sx},{got_sy})");
            }
        }
    }
}
