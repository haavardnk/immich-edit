use crate::edits::{AspectLock, CropRect};

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

pub fn point_in_rotated_source(p: Point, sw: f32, sh: f32, angle_deg: f32) -> bool {
    let bbox = rotated_bbox(sw, sh, angle_deg);
    let a = deg_to_rad(angle_deg);
    let c = a.cos();
    let s = a.sin();
    let cx = bbox.w / 2.0;
    let cy = bbox.h / 2.0;
    let dx = p.x - cx;
    let dy = p.y - cy;
    let ux = dx * c + dy * s;
    let uy = -dx * s + dy * c;
    let hw = sw / 2.0;
    let hh = sh / 2.0;
    ux.abs() <= hw + 1e-3 && uy.abs() <= hh + 1e-3
}

pub fn crop_rect_inside_rotated_source(rect: CropRect, sw: f32, sh: f32, angle_deg: f32) -> bool {
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
        .all(|p| point_in_rotated_source(*p, sw, sh, angle_deg))
}

pub fn largest_inscribed_rect(sw: f32, sh: f32, angle_deg: f32, aspect: f32) -> CropRect {
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
        if crop_rect_inside_rotated_source(rect, sw, sh, angle_deg) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let r = largest_inscribed_rect(100.0, 50.0, 0.0, 2.0);
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
                let r = largest_inscribed_rect(sw, sh, angle, aspect);
                assert!(
                    crop_rect_inside_rotated_source(r, sw, sh, angle),
                    "angle={angle} aspect={aspect} rect={r:?}"
                );
            }
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
}
