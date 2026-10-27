const INVERSE_STEPS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarpPlane {
    pub radial: [f32; 4],
    pub tangential: [f32; 2],
}

impl WarpPlane {
    pub const IDENTITY: Self = Self {
        radial: [1.0, 0.0, 0.0, 0.0],
        tangential: [0.0, 0.0],
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct DngWarp {
    pub planes: [WarpPlane; 3],
    pub center: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarpFrame {
    pub center: [f32; 2],
    pub radius: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EmbeddedLens {
    #[default]
    Absent,
    Shading,
    Warp(DngWarp),
}

impl EmbeddedLens {
    pub fn is_corrected(&self) -> bool {
        *self != Self::Absent
    }

    pub fn warp(&self) -> Option<&DngWarp> {
        match self {
            Self::Warp(warp) => Some(warp),
            Self::Absent | Self::Shading => None,
        }
    }
}

impl DngWarp {
    pub fn is_identity(&self) -> bool {
        self.planes.iter().all(|p| *p == WarpPlane::IDENTITY)
    }

    pub fn is_split(&self) -> bool {
        self.planes[1..].iter().any(|p| *p != self.planes[0])
    }

    pub fn frame(&self, width: usize, height: usize) -> WarpFrame {
        let w = width as f32;
        let h = height as f32;
        let cx = self.center[0] * w;
        let cy = self.center[1] * h;
        let reach_x = cx.max(w - cx);
        let reach_y = cy.max(h - cy);
        WarpFrame {
            center: [cx, cy],
            radius: reach_x.hypot(reach_y).max(f32::MIN_POSITIVE),
        }
    }
}

impl WarpFrame {
    pub fn source(&self, plane: &WarpPlane, x: f32, y: f32) -> [f32; 2] {
        let dx = (x - self.center[0]) / self.radius;
        let dy = (y - self.center[1]) / self.radius;
        let r2 = dx * dx + dy * dy;
        let [k0, k1, k2, k3] = plane.radial;
        let [t0, t1] = plane.tangential;
        let f = k0 + r2 * (k1 + r2 * (k2 + r2 * k3));
        let tx = t0 * 2.0 * dx * dy + t1 * (r2 + 2.0 * dx * dx);
        let ty = t1 * 2.0 * dx * dy + t0 * (r2 + 2.0 * dy * dy);
        [
            self.center[0] + self.radius * (f * dx + tx),
            self.center[1] + self.radius * (f * dy + ty),
        ]
    }

    pub fn source_px(&self, plane: &WarpPlane, x: f32, y: f32) -> [f32; 2] {
        let [sx, sy] = self.source(plane, x + 0.5, y + 0.5);
        [sx - 0.5, sy - 0.5]
    }

    pub fn destination(&self, plane: &WarpPlane, x: f32, y: f32) -> [f32; 2] {
        (0..INVERSE_STEPS).fold([x, y], |[dx, dy], _| {
            let [sx, sy] = self.source(plane, dx, dy);
            [dx + x - sx, dy + y - sy]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warp(plane: WarpPlane, center: [f32; 2]) -> DngWarp {
        DngWarp {
            planes: [plane; 3],
            center,
        }
    }

    #[test]
    fn radius_reaches_the_farthest_corner_from_an_off_centre_origin() {
        let frame = warp(WarpPlane::IDENTITY, [0.25, 0.5]).frame(100, 40);
        assert_eq!(frame.center, [25.0, 20.0]);
        assert!((frame.radius - 75.0f32.hypot(20.0)).abs() < 1e-4);
    }

    #[test]
    fn radial_terms_scale_by_normalised_radius() {
        let plane = WarpPlane {
            radial: [0.9, 0.1, 0.0, 0.0],
            tangential: [0.0, 0.0],
        };
        let frame = warp(plane, [0.5, 0.5]).frame(200, 200);
        let corner = frame.source(&plane, 200.0, 200.0);
        assert!((corner[0] - 200.0).abs() < 1e-3 && (corner[1] - 200.0).abs() < 1e-3);
        let edge = frame.source(&plane, 200.0, 100.0);
        let expected = 100.0 + frame.radius * (0.9 + 0.1 * 0.5) * (100.0 / frame.radius);
        assert!((edge[0] - expected).abs() < 1e-3);
    }

    #[test]
    fn tangential_terms_follow_the_dng_formula() {
        let plane = WarpPlane {
            radial: [1.0, 0.0, 0.0, 0.0],
            tangential: [0.01, 0.02],
        };
        let frame = warp(plane, [0.5, 0.5]).frame(100, 100);
        let [sx, sy] = frame.source(&plane, 80.0, 30.0);
        let m = frame.radius;
        let dx = 30.0 / m;
        let dy = -20.0 / m;
        let r2 = dx * dx + dy * dy;
        let ex = 50.0 + m * (dx + 0.01 * 2.0 * dx * dy + 0.02 * (r2 + 2.0 * dx * dx));
        let ey = 50.0 + m * (dy + 0.02 * 2.0 * dx * dy + 0.01 * (r2 + 2.0 * dy * dy));
        assert!((sx - ex).abs() < 1e-3 && (sy - ey).abs() < 1e-3);
    }

    #[test]
    fn destination_inverts_source() {
        let plane = WarpPlane {
            radial: [0.9844, 0.1333, -0.3011, 0.1486],
            tangential: [0.002, -0.001],
        };
        let frame = warp(plane, [0.48, 0.52]).frame(4000, 2250);
        for (x, y) in [
            (10.5, 7.5),
            (2000.0, 1125.0),
            (3990.5, 2240.5),
            (300.25, 1900.75),
        ] {
            let [sx, sy] = frame.source(&plane, x, y);
            let [dx, dy] = frame.destination(&plane, sx, sy);
            assert!(
                (dx - x).abs() < 0.01 && (dy - y).abs() < 0.01,
                "({x}, {y}) -> ({dx}, {dy})"
            );
        }
    }

    #[test]
    fn split_reports_differing_planes() {
        let mut w = warp(WarpPlane::IDENTITY, [0.5, 0.5]);
        assert!(!w.is_split() && w.is_identity());
        w.planes[2].radial[0] = 1.001;
        assert!(w.is_split() && !w.is_identity());
    }
}
