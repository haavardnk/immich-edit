use super::Face;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::geom::mask_uv_to_display_uv;
use crate::math::smoothstep;
use crate::sensor_sample::geometry_transform;

const CENTRE_SIGMA: f32 = 0.3;
const CENTRE_GAIN: f32 = 1.0;
const FACE_SHARE_MIN: f64 = 0.2;
const FACE_SHARE_MAX: f64 = 0.6;
const FACE_SHARE_PER_AREA: f64 = 6.0;
const FACE_EDGE: f32 = 0.7;

pub(super) struct Weights {
    pub(super) scene: Vec<f32>,
    pub(super) recoverable: Vec<f32>,
    pub(super) face_mass: f64,
}

struct Ellipse {
    centre: [f32; 2],
    radius: [f32; 2],
}

impl Ellipse {
    fn membership(&self, [u, v]: [f32; 2]) -> f32 {
        let du = (u - self.centre[0]) / self.radius[0];
        let dv = (v - self.centre[1]) / self.radius[1];
        1.0 - smoothstep(FACE_EDGE, 1.0, du.hypot(dv))
    }
}

impl Weights {
    pub(super) fn new(
        frame: &RawFrame,
        edits: &Edits,
        faces: &[Face],
        width: usize,
        height: usize,
    ) -> Self {
        let ellipses: Vec<Ellipse> = faces
            .iter()
            .filter_map(|f| display_ellipse(frame, edits, f))
            .collect();
        let n = width * height;
        let uv = |i: usize| {
            [
                ((i % width) as f32 + 0.5) / width as f32,
                ((i / width) as f32 + 0.5) / height as f32,
            ]
        };
        let centre: Vec<f32> = (0..n)
            .map(|i| {
                let [u, v] = uv(i);
                let d2 = (u - 0.5).powi(2) + (v - 0.5).powi(2);
                1.0 + CENTRE_GAIN * (-d2 / (2.0 * CENTRE_SIGMA * CENTRE_SIGMA)).exp()
            })
            .collect();
        let face: Vec<f32> = (0..n)
            .map(|i| {
                ellipses
                    .iter()
                    .map(|e| e.membership(uv(i)))
                    .fold(0.0, f32::max)
            })
            .collect();
        let centre_total: f64 = centre.iter().map(|&c| f64::from(c)).sum();
        let face_total: f64 = face.iter().map(|&f| f64::from(f)).sum();
        let share = if face_total > 0.0 {
            (FACE_SHARE_MIN + FACE_SHARE_PER_AREA * face_total / n as f64).min(FACE_SHARE_MAX)
        } else {
            0.0
        };
        let scene: Vec<f32> = centre
            .iter()
            .zip(&face)
            .map(|(&c, &f)| {
                let from_face = if share > 0.0 {
                    share * f64::from(f) / face_total
                } else {
                    0.0
                };
                ((1.0 - share) * f64::from(c) / centre_total + from_face) as f32
            })
            .collect();
        let face_mass = scene
            .iter()
            .zip(&face)
            .map(|(&w, &f)| f64::from(w) * f64::from(f))
            .sum();
        Self {
            recoverable: scene.clone(),
            scene,
            face_mass,
        }
    }

    pub(super) fn exclude(&mut self, clipped: &[bool]) {
        for ((r, &s), &c) in self.recoverable.iter_mut().zip(&self.scene).zip(clipped) {
            *r = if c { 0.0 } else { s };
        }
    }
}

fn display_ellipse(frame: &RawFrame, edits: &Edits, face: &Face) -> Option<Ellipse> {
    let (ow, oh) = if frame.meta.orientation.0 {
        (frame.meta.height, frame.meta.width)
    } else {
        (frame.meta.width, frame.meta.height)
    };
    let corners = [
        [face.x, face.y],
        [face.x + face.w, face.y],
        [face.x, face.y + face.h],
        [face.x + face.w, face.y + face.h],
    ];
    let mapped = match geometry_transform(edits, ow as u32, oh as u32) {
        Some(t) => corners.map(|c| mask_uv_to_display_uv(&t, c)),
        None => corners,
    };
    let u0 = mapped.iter().map(|c| c[0]).fold(f32::INFINITY, f32::min);
    let u1 = mapped
        .iter()
        .map(|c| c[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let v0 = mapped.iter().map(|c| c[1]).fold(f32::INFINITY, f32::min);
    let v1 = mapped
        .iter()
        .map(|c| c[1])
        .fold(f32::NEG_INFINITY, f32::max);
    if u1 <= 0.0 || u0 >= 1.0 || v1 <= 0.0 || v0 >= 1.0 || u1 <= u0 || v1 <= v0 {
        return None;
    }
    Some(Ellipse {
        centre: [(u0 + u1) * 0.5, (v0 + v1) * 0.5],
        radius: [(u1 - u0) * 0.5, (v1 - v0) * 0.5],
    })
}
