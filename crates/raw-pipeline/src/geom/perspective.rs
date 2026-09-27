use serde::{Deserialize, Serialize};

pub type Mat3 = [f32; 9];

pub const IDENTITY: Mat3 = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
pub const IDENTITY_ROWS: [f32; 12] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

const KEYSTONE_GAIN: f32 = 0.005;
const ASPECT_BASE: f32 = 1.5;
const MIN_W: f32 = 0.05;
const MIN_QUAD_AREA: f32 = 0.05;
const CORNER_LIMIT: f32 = 0.25;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct PerspectiveEdits {
    #[serde(default)]
    pub vertical: f32,
    #[serde(default)]
    pub horizontal: f32,
    #[serde(default)]
    pub aspect: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corners: Option<[[f32; 2]; 4]>,
}

impl PerspectiveEdits {
    pub fn is_identity(&self) -> bool {
        self.vertical.abs() < 1e-4
            && self.horizontal.abs() < 1e-4
            && self.aspect.abs() < 1e-4
            && self
                .corners
                .map(|c| c.iter().flatten().all(|v| v.abs() < 1e-6))
                .unwrap_or(true)
    }

    pub fn clamped(&self) -> Self {
        Self {
            vertical: self.vertical.clamp(-100.0, 100.0),
            horizontal: self.horizontal.clamp(-100.0, 100.0),
            aspect: self.aspect.clamp(-100.0, 100.0),
            corners: self.corners.map(clamp_corners),
        }
    }

    pub fn forward(&self) -> Mat3 {
        self.matrices().0
    }

    pub fn inverse(&self) -> Mat3 {
        self.matrices().1
    }

    fn matrices(&self) -> (Mat3, Mat3) {
        if self.is_identity() {
            return (IDENTITY, IDENTITY);
        }
        let c = self.clamped();
        let raw = mat3_mul(&centered_to_uv(&c.params_matrix()), &c.corner_matrix());
        let forward = mat3_mul(&fit_to_frame(&raw), &raw);
        let Some(inverse) = mat3_inverse(&forward) else {
            return (IDENTITY, IDENTITY);
        };
        if unit_square_corners()
            .iter()
            .any(|p| homogeneous_w(&inverse, *p).abs() < MIN_W)
        {
            return (IDENTITY, IDENTITY);
        }
        (forward, inverse)
    }

    fn params_matrix(&self) -> Mat3 {
        let kv = self.vertical * KEYSTONE_GAIN;
        let kh = self.horizontal * KEYSTONE_GAIN;
        let a = ASPECT_BASE.powf(self.aspect / 100.0);
        let keystone_v = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, kv, 1.0];
        let keystone_h = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, kh, 0.0, 1.0];
        let aspect = [a, 0.0, 0.0, 0.0, 1.0 / a, 0.0, 0.0, 0.0, 1.0];
        let m = mat3_mul(&aspect, &keystone_v);
        mat3_mul(&m, &keystone_h)
    }

    fn corner_matrix(&self) -> Mat3 {
        let Some(offsets) = self.corners else {
            return IDENTITY;
        };
        let base = unit_square_corners();
        let mut quad = [[0.0f32; 2]; 4];
        for i in 0..4 {
            quad[i][0] = base[i][0] + offsets[i][0];
            quad[i][1] = base[i][1] + offsets[i][1];
        }
        if !quad_is_usable(&quad) {
            return IDENTITY;
        }
        square_to_quad(&quad).unwrap_or(IDENTITY)
    }
}

pub fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [0.0f32; 9];
    for r in 0..3 {
        for c in 0..3 {
            out[r * 3 + c] = a[r * 3] * b[c] + a[r * 3 + 1] * b[3 + c] + a[r * 3 + 2] * b[6 + c];
        }
    }
    out
}

pub fn mat3_inverse(m: &Mat3) -> Option<Mat3> {
    let c00 = m[4] * m[8] - m[5] * m[7];
    let c01 = m[5] * m[6] - m[3] * m[8];
    let c02 = m[3] * m[7] - m[4] * m[6];
    let det = m[0] * c00 + m[1] * c01 + m[2] * c02;
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        c00 * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        c01 * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        c02 * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ])
}

pub fn mat3_apply(m: &Mat3, p: [f32; 2]) -> [f32; 2] {
    let w = homogeneous_w(m, p);
    if w.abs() < 1e-9 {
        return p;
    }
    let inv_w = 1.0 / w;
    [
        (m[0] * p[0] + m[1] * p[1] + m[2]) * inv_w,
        (m[3] * p[0] + m[4] * p[1] + m[5]) * inv_w,
    ]
}

pub fn mat3_rows(m: &Mat3) -> [f32; 12] {
    [
        m[0], m[1], m[2], 0.0, m[3], m[4], m[5], 0.0, m[6], m[7], m[8], 0.0,
    ]
}

pub fn square_to_quad(quad: &[[f32; 2]; 4]) -> Option<Mat3> {
    let [p0, p1, p2, p3] = *quad;
    let sx = p0[0] - p1[0] + p2[0] - p3[0];
    let sy = p0[1] - p1[1] + p2[1] - p3[1];
    if sx.abs() < 1e-9 && sy.abs() < 1e-9 {
        return Some([
            p1[0] - p0[0],
            p3[0] - p0[0],
            p0[0],
            p1[1] - p0[1],
            p3[1] - p0[1],
            p0[1],
            0.0,
            0.0,
            1.0,
        ]);
    }
    let dx1 = p1[0] - p2[0];
    let dx2 = p3[0] - p2[0];
    let dy1 = p1[1] - p2[1];
    let dy2 = p3[1] - p2[1];
    let den = dx1 * dy2 - dx2 * dy1;
    if den.abs() < 1e-12 {
        return None;
    }
    let g = (sx * dy2 - dx2 * sy) / den;
    let h = (dx1 * sy - sx * dy1) / den;
    Some([
        p1[0] - p0[0] + g * p1[0],
        p3[0] - p0[0] + h * p3[0],
        p0[0],
        p1[1] - p0[1] + g * p1[1],
        p3[1] - p0[1] + h * p3[1],
        p0[1],
        g,
        h,
        1.0,
    ])
}

fn fit_to_frame(m: &Mat3) -> Mat3 {
    let pts = unit_square_corners().map(|p| mat3_apply(m, p));
    let min_x = pts.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let max_x = pts.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_y = pts.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let max_y = pts.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
    let w = (max_x - min_x).max(1e-6);
    let h = (max_y - min_y).max(1e-6);
    let s = (1.0 / w).min(1.0 / h).min(1.0);
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    [s, 0.0, 0.5 - s * cx, 0.0, s, 0.5 - s * cy, 0.0, 0.0, 1.0]
}

fn homogeneous_w(m: &Mat3, p: [f32; 2]) -> f32 {
    m[6] * p[0] + m[7] * p[1] + m[8]
}

fn unit_square_corners() -> [[f32; 2]; 4] {
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

fn centered_to_uv(m: &Mat3) -> Mat3 {
    let to_uv = [1.0, 0.0, 0.5, 0.0, 1.0, 0.5, 0.0, 0.0, 1.0];
    let to_centered = [1.0, 0.0, -0.5, 0.0, 1.0, -0.5, 0.0, 0.0, 1.0];
    mat3_mul(&mat3_mul(&to_uv, m), &to_centered)
}

fn clamp_corners(offsets: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    let mut out = [[0.0f32; 2]; 4];
    for i in 0..4 {
        for axis in 0..2 {
            out[i][axis] = offsets[i][axis].clamp(-CORNER_LIMIT, CORNER_LIMIT);
        }
    }
    out
}

fn quad_is_usable(quad: &[[f32; 2]; 4]) -> bool {
    let mut area = 0.0f32;
    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let c = quad[(i + 2) % 4];
        area += a[0] * b[1] - b[0] * a[1];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if sign == 0.0 {
            sign = cross;
        } else if cross * sign < 0.0 {
            return false;
        }
    }
    (area * 0.5) >= MIN_QUAD_AREA
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        let p = PerspectiveEdits::default();
        assert!(p.is_identity());
        assert_eq!(p.forward(), IDENTITY);
        assert_eq!(p.inverse(), IDENTITY);
    }

    #[test]
    fn inverse_round_trips() {
        let cases = [
            PerspectiveEdits {
                vertical: 60.0,
                ..Default::default()
            },
            PerspectiveEdits {
                horizontal: -45.0,
                aspect: 30.0,
                ..Default::default()
            },
            PerspectiveEdits {
                vertical: 25.0,
                horizontal: 15.0,
                aspect: -20.0,
                ..Default::default()
            },
            PerspectiveEdits {
                corners: Some([[0.05, 0.08], [-0.1, 0.02], [0.03, -0.06], [0.07, -0.04]]),
                ..Default::default()
            },
        ];
        for p in cases {
            let f = p.forward();
            let i = p.inverse();
            assert_ne!(f, IDENTITY, "{p:?} produced identity");
            for uv in [[0.0, 0.0], [0.25, 0.75], [0.5, 0.5], [1.0, 1.0]] {
                let back = mat3_apply(&i, mat3_apply(&f, uv));
                assert!(
                    (back[0] - uv[0]).abs() < 1e-3 && (back[1] - uv[1]).abs() < 1e-3,
                    "{p:?} uv={uv:?} back={back:?}"
                );
            }
        }
    }

    #[test]
    fn aspect_stretches_about_center() {
        let p = PerspectiveEdits {
            aspect: 100.0,
            ..Default::default()
        };
        let center = mat3_apply(&p.forward(), [0.5, 0.5]);
        assert!((center[0] - 0.5).abs() < 1e-5 && (center[1] - 0.5).abs() < 1e-5);
        let corner = mat3_apply(&p.forward(), [1.0, 1.0]);
        assert!((corner[0] - 1.0).abs() < 1e-5 && (corner[1] - 0.5 - 2.0 / 9.0).abs() < 1e-5);
    }

    #[test]
    fn warped_quad_always_fits_the_frame() {
        let cases = [
            PerspectiveEdits {
                vertical: 100.0,
                ..Default::default()
            },
            PerspectiveEdits {
                vertical: 100.0,
                horizontal: 100.0,
                aspect: 40.0,
                ..Default::default()
            },
            PerspectiveEdits {
                corners: Some([[-0.25, -0.25], [0.25, -0.25], [0.25, 0.25], [-0.25, 0.25]]),
                ..Default::default()
            },
        ];
        for p in cases {
            let f = p.forward();
            for uv in unit_square_corners() {
                let out = mat3_apply(&f, uv);
                assert!(
                    out[0] >= -1e-4 && out[0] <= 1.0 + 1e-4,
                    "{p:?} uv={uv:?} out={out:?}"
                );
                assert!(
                    out[1] >= -1e-4 && out[1] <= 1.0 + 1e-4,
                    "{p:?} uv={uv:?} out={out:?}"
                );
            }
        }
    }

    #[test]
    fn vertical_keystone_narrows_one_edge() {
        let p = PerspectiveEdits {
            vertical: 100.0,
            ..Default::default()
        };
        let f = p.forward();
        let top_left = mat3_apply(&f, [0.0, 0.0]);
        let top_right = mat3_apply(&f, [1.0, 0.0]);
        let bottom_left = mat3_apply(&f, [0.0, 1.0]);
        let bottom_right = mat3_apply(&f, [1.0, 1.0]);
        let top_width = top_right[0] - top_left[0];
        let bottom_width = bottom_right[0] - bottom_left[0];
        assert!(bottom_width < top_width, "{bottom_width} !< {top_width}");
    }

    #[test]
    fn mirrored_corners_fall_back_to_identity() {
        let quad = [[0.9, 0.9], [0.1, 0.9], [0.1, 0.1], [0.9, 0.1]];
        let flipped = [quad[0], quad[3], quad[2], quad[1]];
        assert!(!quad_is_usable(&flipped));
    }

    #[test]
    fn collapsed_corners_fall_back_to_identity() {
        let quad = [[0.4, 0.45], [0.6, 0.45], [0.6, 0.55], [0.4, 0.55]];
        assert!(!quad_is_usable(&quad));
    }

    #[test]
    fn clamped_limits_ranges() {
        let p = PerspectiveEdits {
            vertical: 500.0,
            horizontal: -500.0,
            aspect: 900.0,
            corners: Some([[9.0, 9.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]),
        }
        .clamped();
        assert_eq!(p.vertical, 100.0);
        assert_eq!(p.horizontal, -100.0);
        assert_eq!(p.aspect, 100.0);
        assert_eq!(p.corners.unwrap()[0], [0.25, 0.25]);
    }
}
