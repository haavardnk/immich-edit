use crate::edits::{AspectLock, CropRect};
use crate::frame::OrientFlips;
use crate::perspective::{IDENTITY, Mat3, mat3_apply};

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

pub fn rotated_bbox(sw: f32, sh: f32, angle_deg: f32) -> Size {
    let a = deg_to_rad(angle_deg);
    let c = a.cos().abs();
    let s = a.sin().abs();
    Size {
        w: sw * c + sh * s,
        h: sw * s + sh * c,
    }
}

pub fn scale_to_max(w: u32, h: u32, max_edge: u32) -> (u32, u32) {
    if w <= max_edge && h <= max_edge {
        return (w.max(1), h.max(1));
    }
    let scale = max_edge as f64 / w.max(h) as f64;
    let nw = (w as f64 * scale).round() as u32;
    let nh = (h as f64 * scale).round() as u32;
    (nw.max(1), nh.max(1))
}

pub fn display_crop_px(
    orientation: OrientFlips,
    edits: &crate::edits::Edits,
    src_dims: (u32, u32),
) -> (u32, u32) {
    let (sensor_w, sensor_h) = src_dims;
    let (display_w, display_h) = if orientation.0 {
        (sensor_h, sensor_w)
    } else {
        (sensor_w, sensor_h)
    };
    let (oriented_w, oriented_h) = match edits.geometry.rotate {
        90 | 270 => (display_h, display_w),
        _ => (display_w, display_h),
    };
    let crop = edits.geometry.crop.unwrap_or(CropRect::full());
    let bbox = rotated_bbox(
        oriented_w as f32,
        oriented_h as f32,
        edits.geometry.rotate_angle,
    );
    let crop_w_px = (crop.w * bbox.w).round().max(1.0) as u32;
    let crop_h_px = (crop.h * bbox.h).round().max(1.0) as u32;
    (crop_w_px, crop_h_px)
}

pub fn display_out_dims(
    orientation: OrientFlips,
    edits: &crate::edits::Edits,
    src_dims: (u32, u32),
    max_edge: u32,
) -> (u32, u32) {
    let (crop_w_px, crop_h_px) = display_crop_px(orientation, edits, src_dims);
    scale_to_max(crop_w_px, crop_h_px, max_edge)
}

pub fn source_quad_in_bbox(sw: f32, sh: f32, angle_deg: f32) -> [Point; 4] {
    let a = deg_to_rad(angle_deg);
    let c = a.cos();
    let s = a.sin();
    let bbox = rotated_bbox(sw, sh, angle_deg);
    let cx = bbox.w / 2.0;
    let cy = bbox.h / 2.0;
    let hw = sw / 2.0;
    let hh = sh / 2.0;
    let corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)];
    corners.map(|(x, y)| Point {
        x: cx + x * c - y * s,
        y: cy + x * s + y * c,
    })
}

pub fn aspect_ratio_for(aspect: AspectLock, sw: f32, sh: f32) -> Option<f32> {
    match aspect {
        AspectLock::Free => None,
        AspectLock::Original => Some(sw / sh),
        AspectLock::Ratio { num, den } => {
            if num == 0 || den == 0 {
                None
            } else {
                Some(num as f32 / den as f32)
            }
        }
    }
}

pub fn point_in_warped_source(
    p: Point,
    sw: f32,
    sh: f32,
    angle_deg: f32,
    persp_inv: &Mat3,
) -> bool {
    let bbox = rotated_bbox(sw, sh, angle_deg);
    let uv = display_uv_to_oriented_uv(
        CropRect::full(),
        bbox,
        sw,
        sh,
        angle_deg,
        persp_inv,
        [p.x / bbox.w, p.y / bbox.h],
    );
    let eps_u = 1e-3 / sw;
    let eps_v = 1e-3 / sh;
    uv[0] >= -eps_u && uv[0] <= 1.0 + eps_u && uv[1] >= -eps_v && uv[1] <= 1.0 + eps_v
}

pub fn crop_rect_inside_warped_source(
    rect: CropRect,
    sw: f32,
    sh: f32,
    angle_deg: f32,
    persp_inv: &Mat3,
) -> bool {
    let bbox = rotated_bbox(sw, sh, angle_deg);
    let x0 = rect.x * bbox.w;
    let y0 = rect.y * bbox.h;
    let x1 = (rect.x + rect.w) * bbox.w;
    let y1 = (rect.y + rect.h) * bbox.h;
    let corners = [
        Point { x: x0, y: y0 },
        Point { x: x1, y: y0 },
        Point { x: x1, y: y1 },
        Point { x: x0, y: y1 },
    ];
    corners
        .iter()
        .all(|p| point_in_warped_source(*p, sw, sh, angle_deg, persp_inv))
}

pub fn largest_inscribed_rect(
    sw: f32,
    sh: f32,
    angle_deg: f32,
    aspect: f32,
    persp_inv: &Mat3,
) -> CropRect {
    let bbox = rotated_bbox(sw, sh, angle_deg);
    let target_aspect = aspect.max(1e-6);
    let bbox_aspect = bbox.w / bbox.h;
    let (base_w, base_h) = if bbox_aspect >= target_aspect {
        (bbox.h * target_aspect, bbox.h)
    } else {
        (bbox.w, bbox.w / target_aspect)
    };
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let w_px = base_w * mid;
        let h_px = base_h * mid;
        let nx = (bbox.w - w_px) / 2.0 / bbox.w;
        let ny = (bbox.h - h_px) / 2.0 / bbox.h;
        let rect = CropRect {
            x: nx,
            y: ny,
            w: w_px / bbox.w,
            h: h_px / bbox.h,
        };
        if crop_rect_inside_warped_source(rect, sw, sh, angle_deg, persp_inv) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let w_px = base_w * lo;
    let h_px = base_h * lo;
    let nx = (bbox.w - w_px) / 2.0 / bbox.w;
    let ny = (bbox.h - h_px) / 2.0 / bbox.h;
    CropRect {
        x: nx.clamp(0.0, 1.0),
        y: ny.clamp(0.0, 1.0),
        w: (w_px / bbox.w).clamp(0.0, 1.0),
        h: (h_px / bbox.h).clamp(0.0, 1.0),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeometryTransform {
    pub input_w: u32,
    pub input_h: u32,
    pub rotate_quarter: u16,
    pub flip_h: bool,
    pub flip_v: bool,
    pub angle_deg: f32,
    pub crop: CropRect,
    pub perspective_forward: Mat3,
    pub perspective_inverse: Mat3,
    pub output_w: u32,
    pub output_h: u32,
}

impl GeometryTransform {
    pub fn is_identity(&self) -> bool {
        self.rotate_quarter == 0
            && !self.flip_h
            && !self.flip_v
            && self.angle_deg.abs() < 1e-4
            && self.crop.is_full()
            && self.perspective_forward == IDENTITY
    }
    pub fn oriented_size(&self) -> (u32, u32) {
        match self.rotate_quarter {
            90 | 270 => (self.input_h, self.input_w),
            _ => (self.input_w, self.input_h),
        }
    }
    pub fn bbox(&self) -> Size {
        let (ow, oh) = self.oriented_size();
        rotated_bbox(ow as f32, oh as f32, self.angle_deg)
    }
}

fn ortho_forward(rot: u16, flip_h: bool, flip_v: bool, mu: f32, mv: f32) -> (f32, f32) {
    let (mut u, mut v) = match rot {
        90 => (1.0 - mv, mu),
        180 => (1.0 - mu, 1.0 - mv),
        270 => (mv, 1.0 - mu),
        _ => (mu, mv),
    };
    if flip_h {
        u = 1.0 - u;
    }
    if flip_v {
        v = 1.0 - v;
    }
    (u, v)
}

fn ortho_inverse(rot: u16, flip_h: bool, flip_v: bool, u: f32, v: f32) -> (f32, f32) {
    let mut uu = u;
    let mut vv = v;
    if flip_h {
        uu = 1.0 - uu;
    }
    if flip_v {
        vv = 1.0 - vv;
    }
    match rot {
        90 => (vv, 1.0 - uu),
        180 => (1.0 - uu, 1.0 - vv),
        270 => (1.0 - vv, uu),
        _ => (uu, vv),
    }
}

pub fn display_uv_to_oriented_uv(
    crop: CropRect,
    bbox: Size,
    ow: f32,
    oh: f32,
    angle_deg: f32,
    persp_inv: &Mat3,
    uv: [f32; 2],
) -> [f32; 2] {
    let a = deg_to_rad(angle_deg);
    let cos_a = a.cos();
    let sin_a = a.sin();
    let bx_rel = crop.x + uv[0] * crop.w;
    let by_rel = crop.y + uv[1] * crop.h;
    let cx_px = (bx_rel - 0.5) * bbox.w;
    let cy_px = (by_rel - 0.5) * bbox.h;
    let sx_px = cx_px * cos_a + cy_px * sin_a;
    let sy_px = -cx_px * sin_a + cy_px * cos_a;
    mat3_apply(persp_inv, [sx_px / ow + 0.5, sy_px / oh + 0.5])
}

pub fn oriented_uv_to_display_uv(
    crop: CropRect,
    bbox: Size,
    ow: f32,
    oh: f32,
    angle_deg: f32,
    persp_fwd: &Mat3,
    uv: [f32; 2],
) -> [f32; 2] {
    let a = deg_to_rad(angle_deg);
    let cos_a = a.cos();
    let sin_a = a.sin();
    let warped = mat3_apply(persp_fwd, uv);
    let sx_px = (warped[0] - 0.5) * ow;
    let sy_px = (warped[1] - 0.5) * oh;
    let cx_px = sx_px * cos_a - sy_px * sin_a;
    let cy_px = sx_px * sin_a + sy_px * cos_a;
    let bx_rel = cx_px / bbox.w + 0.5;
    let by_rel = cy_px / bbox.h + 0.5;
    let crop_w = crop.w.max(1e-9);
    let crop_h = crop.h.max(1e-9);
    [(bx_rel - crop.x) / crop_w, (by_rel - crop.y) / crop_h]
}

pub fn display_uv_to_mask_uv(t: &GeometryTransform, uv: [f32; 2]) -> [f32; 2] {
    if t.is_identity() {
        return uv;
    }
    let (ow, oh) = t.oriented_size();
    let o = display_uv_to_oriented_uv(
        t.crop,
        t.bbox(),
        ow as f32,
        oh as f32,
        t.angle_deg,
        &t.perspective_inverse,
        uv,
    );
    let (mu, mv) = ortho_inverse(t.rotate_quarter, t.flip_h, t.flip_v, o[0], o[1]);
    [mu, mv]
}

pub fn mask_uv_to_display_uv(t: &GeometryTransform, uv: [f32; 2]) -> [f32; 2] {
    if t.is_identity() {
        return uv;
    }
    let (ow, oh) = t.oriented_size();
    let (u_o, v_o) = ortho_forward(t.rotate_quarter, t.flip_h, t.flip_v, uv[0], uv[1]);
    oriented_uv_to_display_uv(
        t.crop,
        t.bbox(),
        ow as f32,
        oh as f32,
        t.angle_deg,
        &t.perspective_forward,
        [u_o, v_o],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_to_max_cases() {
        let cases = [
            ((100, 50, 200), (100, 50)),
            ((4000, 3000, 1000), (1000, 750)),
            ((3000, 4000, 1000), (750, 1000)),
            ((6000, 1000, 512), (512, 85)),
        ];
        for ((w, h, max), want) in cases {
            assert_eq!(scale_to_max(w, h, max), want, "{w}x{h} @ {max}");
        }
    }

    #[test]
    fn display_out_dims_follows_orientation_and_crop() {
        let mut edits = crate::edits::Edits::default();
        assert_eq!(
            display_out_dims((false, false, false), &edits, (4000, 3000), 512),
            (512, 384)
        );
        assert_eq!(
            display_out_dims((true, false, false), &edits, (4000, 3000), 512),
            (384, 512)
        );

        edits.geometry.rotate = 90;
        assert_eq!(
            display_out_dims((false, false, false), &edits, (4000, 3000), 512),
            (384, 512)
        );

        edits.geometry.rotate = 0;
        edits.geometry.crop = Some(CropRect {
            x: 0.0,
            y: 0.0,
            w: 0.5,
            h: 0.5,
        });
        assert_eq!(
            display_out_dims((false, false, false), &edits, (4000, 3000), 512),
            (512, 384)
        );
        assert_eq!(
            display_crop_px((false, false, false), &edits, (4000, 3000)),
            (2000, 1500)
        );
    }

    #[test]
    fn bbox_zero_angle() {
        let b = rotated_bbox(100.0, 50.0, 0.0);
        assert!((b.w - 100.0).abs() < 1e-3);
        assert!((b.h - 50.0).abs() < 1e-3);
    }

    #[test]
    fn bbox_90_swaps() {
        let b = rotated_bbox(100.0, 50.0, 90.0);
        assert!((b.w - 50.0).abs() < 1e-3);
        assert!((b.h - 100.0).abs() < 1e-3);
    }

    #[test]
    fn inscribed_identity_at_zero() {
        let r = largest_inscribed_rect(100.0, 50.0, 0.0, 2.0, &IDENTITY);
        let bbox = rotated_bbox(100.0, 50.0, 0.0);
        let w_px = r.w * bbox.w;
        let h_px = r.h * bbox.h;
        assert!((w_px - 100.0).abs() < 1e-2, "w={w_px}");
        assert!((h_px - 50.0).abs() < 1e-2, "h={h_px}");
    }

    #[test]
    fn inscribed_inside_rotated_source() {
        for &angle in &[5.0_f32, 10.0, 20.0, 30.0, -15.0, 45.0] {
            for &aspect in &[1.0_f32, 4.0 / 3.0, 3.0 / 4.0, 16.0 / 9.0] {
                let sw = 1200.0;
                let sh = 800.0;
                let r = largest_inscribed_rect(sw, sh, angle, aspect, &IDENTITY);
                assert!(
                    crop_rect_inside_warped_source(r, sw, sh, angle, &IDENTITY),
                    "angle={angle} aspect={aspect} rect={r:?}"
                );
            }
        }
    }

    #[test]
    fn inscribed_inside_warped_source() {
        let p = crate::perspective::PerspectiveEdits {
            vertical: 40.0,
            horizontal: -20.0,
            ..Default::default()
        };
        let inv = p.inverse();
        for &angle in &[0.0_f32, 8.0, -12.0] {
            let sw = 1200.0;
            let sh = 800.0;
            let r = largest_inscribed_rect(sw, sh, angle, 1.5, &inv);
            assert!(r.w > 0.1 && r.h > 0.1, "angle={angle} rect={r:?}");
            assert!(
                crop_rect_inside_warped_source(r, sw, sh, angle, &inv),
                "angle={angle} rect={r:?}"
            );
        }
    }

    #[test]
    fn aspect_resolves() {
        assert_eq!(aspect_ratio_for(AspectLock::Free, 100.0, 50.0), None);
        assert_eq!(
            aspect_ratio_for(AspectLock::Original, 100.0, 50.0),
            Some(2.0)
        );
        assert_eq!(
            aspect_ratio_for(AspectLock::Ratio { num: 16, den: 9 }, 100.0, 50.0),
            Some(16.0 / 9.0)
        );
    }

    fn xform(
        rot: u16,
        flip_h: bool,
        flip_v: bool,
        angle: f32,
        crop: CropRect,
    ) -> GeometryTransform {
        xform_with(rot, flip_h, flip_v, angle, crop, IDENTITY, IDENTITY)
    }

    fn xform_with(
        rot: u16,
        flip_h: bool,
        flip_v: bool,
        angle: f32,
        crop: CropRect,
        persp_fwd: Mat3,
        persp_inv: Mat3,
    ) -> GeometryTransform {
        let (iw, ih) = (1200u32, 800u32);
        let (ow, oh) = match rot {
            90 | 270 => (ih, iw),
            _ => (iw, ih),
        };
        let bbox = rotated_bbox(ow as f32, oh as f32, angle);
        let out_w = (crop.w * bbox.w).round().max(1.0) as u32;
        let out_h = (crop.h * bbox.h).round().max(1.0) as u32;
        GeometryTransform {
            input_w: iw,
            input_h: ih,
            rotate_quarter: rot,
            flip_h,
            flip_v,
            angle_deg: angle,
            crop,
            perspective_forward: persp_fwd,
            perspective_inverse: persp_inv,
            output_w: out_w,
            output_h: out_h,
        }
    }

    #[test]
    fn geom_identity_passthrough() {
        let t = xform(0, false, false, 0.0, CropRect::full());
        let uv = [0.37, 0.81];
        let m = display_uv_to_mask_uv(&t, uv);
        assert!((m[0] - uv[0]).abs() < 1e-7 && (m[1] - uv[1]).abs() < 1e-7);
    }

    #[test]
    fn geom_round_trip() {
        let crops = [
            CropRect::full(),
            CropRect {
                x: 0.1,
                y: 0.15,
                w: 0.7,
                h: 0.6,
            },
        ];
        for &rot in &[0u16, 90, 180, 270] {
            for &flip_h in &[false, true] {
                for &flip_v in &[false, true] {
                    for &angle in &[0.0f32, 5.0, -7.5] {
                        for &crop in &crops {
                            let t = xform(rot, flip_h, flip_v, angle, crop);
                            for uv in [[0.1, 0.2], [0.5, 0.5], [0.85, 0.9]] {
                                let m = display_uv_to_mask_uv(&t, uv);
                                let d = mask_uv_to_display_uv(&t, m);
                                if (d[0] - uv[0]).abs() > 1e-4 || (d[1] - uv[1]).abs() > 1e-4 {
                                    panic!(
                                        "round trip rot={rot} fh={flip_h} fv={flip_v} a={angle} crop={crop:?} uv={uv:?} back={d:?} mask={m:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn geom_round_trip_with_perspective() {
        let p = crate::perspective::PerspectiveEdits {
            vertical: 55.0,
            horizontal: -30.0,
            aspect: 20.0,
            ..Default::default()
        };
        for &rot in &[0u16, 90, 270] {
            for &angle in &[0.0f32, 6.0] {
                let t = xform_with(
                    rot,
                    false,
                    true,
                    angle,
                    CropRect {
                        x: 0.1,
                        y: 0.05,
                        w: 0.75,
                        h: 0.8,
                    },
                    p.forward(),
                    p.inverse(),
                );
                assert!(!t.is_identity());
                for uv in [[0.2, 0.3], [0.5, 0.5], [0.9, 0.7]] {
                    let m = display_uv_to_mask_uv(&t, uv);
                    let d = mask_uv_to_display_uv(&t, m);
                    assert!(
                        (d[0] - uv[0]).abs() < 1e-3 && (d[1] - uv[1]).abs() < 1e-3,
                        "rot={rot} angle={angle} uv={uv:?} back={d:?}"
                    );
                }
            }
        }
    }
}
