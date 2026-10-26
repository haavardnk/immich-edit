use crate::math::smoothstep;

const MAX_BLUR_FRACTION: f32 = 0.02;
const GRID_RADIUS: f32 = 16.0;
pub const HIGHLIGHT_GAIN: f32 = 8.0;
pub const HIGHLIGHT_LO: f32 = 0.5;
pub const HIGHLIGHT_HI: f32 = 1.0;
pub const UPSAMPLE_SHARPNESS: f32 = 2.0;
pub const APERTURE_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlurScale {
    pub radius: f32,
    pub ds: u32,
    pub reach: i32,
}

impl BlurScale {
    pub fn new(amount: f32, (w, h): (u32, u32)) -> Self {
        let radius = (amount / 100.0) * MAX_BLUR_FRACTION * w.max(h) as f32;
        let ds = ((radius / GRID_RADIUS).ceil().max(1.0) as u32).next_power_of_two();
        Self {
            radius,
            ds,
            reach: (radius / ds as f32 + 0.5).ceil() as i32,
        }
    }

    pub fn margin(&self) -> u32 {
        self.radius.ceil() as u32 + 3 * self.ds + 2
    }
}

#[inline(always)]
pub fn aperture_distance(dx: f32, dy: f32, blades: u32) -> f32 {
    let len = (dx * dx + dy * dy).sqrt();
    if blades == 0 || len == 0.0 {
        return len;
    }
    let sector = std::f32::consts::TAU / blades as f32;
    let theta = dy.atan2(dx) - APERTURE_ROTATION;
    let phi = theta - sector * (theta / sector).round();
    len * phi.cos() / (std::f32::consts::PI / blades as f32).cos()
}

#[inline(always)]
pub fn highlight_weight(rgb: [f32; 3], boost: f32) -> f32 {
    let peak = rgb[0].max(rgb[1]).max(rgb[2]);
    1.0 + boost * HIGHLIGHT_GAIN * smoothstep(HIGHLIGHT_LO, HIGHLIGHT_HI, peak)
}

#[inline(always)]
pub fn sender_reach(receiver: f32, sender: f32) -> f32 {
    if sender < receiver {
        sender.abs().min(receiver.abs())
    } else {
        sender.abs()
    }
}

#[inline(always)]
pub fn gather_weight(reach: f32, distance: f32) -> f32 {
    let coverage = (reach - distance + 0.5).clamp(0.0, 1.0);
    let area = reach.max(0.5);
    coverage / (area * area)
}

#[inline(always)]
pub fn upsample_weight(cell: f32, pixel: f32) -> f32 {
    1.0 / (1.0 + UPSAMPLE_SHARPNESS * (cell - pixel).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_keeps_its_reach_near_the_target_radius() {
        let preview = BlurScale::new(100.0, (2048, 1365));
        assert_eq!(preview.ds, 4);
        let full = BlurScale::new(100.0, (6000, 4000));
        assert_eq!(full.ds, 8);
        assert!(full.reach as f32 <= GRID_RADIUS + 1.0);
        let small = BlurScale::new(10.0, (1000, 800));
        assert_eq!(small.ds, 1);
    }

    #[test]
    fn a_hexagon_reaches_its_corners_and_stops_short_on_its_faces() {
        let face = aperture_distance(0.0, 1.0, 6);
        let corner = aperture_distance(1.0, 0.0, 6);
        assert!((corner - 1.0).abs() < 1e-5, "{corner}");
        assert!(face > 1.1, "{face}");
        assert_eq!(aperture_distance(3.0, 4.0, 0), 5.0);
    }

    #[test]
    fn a_farther_sender_never_covers_a_nearer_receiver() {
        assert_eq!(gather_weight(sender_reach(0.0, -6.0), 2.0), 0.0);
        assert!(gather_weight(sender_reach(0.0, 6.0), 2.0) > 0.0);
        assert!(gather_weight(sender_reach(-6.0, -6.0), 2.0) > 0.0);
    }
}
