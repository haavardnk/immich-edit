use crate::dehaze::DehazeGrid;
use crate::edits::Edits;
use crate::frame::FrameMeta;
use crate::geom::display_uv_to_mask_uv;
use crate::presence::{presence_pyramid_levels, presence_radii};
use crate::sensor_sample::geometry_transform;

const CUBIC_SUPPORT: u32 = 2;
const DISPLAY_CORNERS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowRect {
    pub origin: (u32, u32),
    pub dims: (u32, u32),
}

pub fn window_rect(meta: &FrameMeta, edits: &Edits, full: (u32, u32)) -> Option<WindowRect> {
    let (fw, fh) = full;
    let transpose = meta.orientation.0;
    let oriented = if transpose { (fh, fw) } else { (fw, fh) };
    let transform = geometry_transform(edits, oriented.0, oriented.1);
    let sensor = DISPLAY_CORNERS.map(|uv| {
        let [mu, mv] = match &transform {
            Some(t) => display_uv_to_mask_uv(t, uv),
            None => uv,
        };
        let (mut su, mut sv) = if transpose { (mv, mu) } else { (mu, mv) };
        if meta.orientation.2 {
            sv = 1.0 - sv;
        }
        if meta.orientation.1 {
            su = 1.0 - su;
        }
        (su.clamp(0.0, 1.0), sv.clamp(0.0, 1.0))
    });
    let (u0, u1) = bounds(sensor.map(|p| p.0));
    let (v0, v1) = bounds(sensor.map(|p| p.1));
    let (align, margin) = spatial_support(full);
    let start = |t: f32, n: u32| {
        let px = (t * n as f32).floor() as u32;
        px.saturating_sub(margin) / align * align
    };
    let end = |t: f32, n: u32| {
        let px = (t * n as f32).ceil() as u32 + margin;
        (px.div_ceil(align) * align).min(n)
    };
    let (x0, x1) = (start(u0, fw), end(u1, fw));
    let (y0, y1) = (start(v0, fh), end(v1, fh));
    if x0 == 0 && y0 == 0 && x1 == fw && y1 == fh {
        return None;
    }
    Some(WindowRect {
        origin: (x0, y0),
        dims: (x1 - x0, y1 - y0),
    })
}

pub fn crop_rgb<T: Copy>(rgb: &[T], width: u32, rect: WindowRect) -> Vec<T> {
    let (x0, y0) = (rect.origin.0 as usize, rect.origin.1 as usize);
    let (w, h) = (rect.dims.0 as usize, rect.dims.1 as usize);
    let stride = width as usize * 3;
    (y0..y0 + h)
        .flat_map(|y| {
            rgb[y * stride + x0 * 3..y * stride + (x0 + w) * 3]
                .iter()
                .copied()
        })
        .collect()
}

fn bounds(values: [f32; 4]) -> (f32, f32) {
    values
        .into_iter()
        .fold((1.0, 0.0), |(lo, hi), v| (lo.min(v), hi.max(v)))
}

fn spatial_support(full: (u32, u32)) -> (u32, u32) {
    let radii = presence_radii(full.0, full.1);
    let top = presence_pyramid_levels(full.0, full.1, radii) - 1;
    let presence = 2u32 << top;
    let dehaze = DehazeGrid::for_dims(full);
    let align = (1u32 << top).max(dehaze.scale);
    (align, dehaze.support() + 2 * presence + CUBIC_SUPPORT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::CropRect;

    fn meta(orientation: (bool, bool, bool)) -> FrameMeta {
        FrameMeta {
            width: 6000,
            height: 4000,
            wb_coeffs: [1.0; 4],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation,
            is_raw: true,
            capture_sigma: None,
            model: String::new(),
        }
    }

    fn cropped(x: f32, y: f32, w: f32, h: f32) -> Edits {
        let mut edits = Edits::default();
        edits.geometry.crop = Some(CropRect { x, y, w, h });
        edits
    }

    #[test]
    fn a_full_view_needs_no_window() {
        let full = (6000, 4000);
        if window_rect(&meta((false, false, false)), &Edits::default(), full).is_some() {
            panic!("the whole frame was windowed");
        }
    }

    #[test]
    fn a_window_covers_the_region_with_an_aligned_margin() {
        let full = (6000, 4000);
        let (align, margin) = spatial_support(full);
        let edits = cropped(0.4, 0.4, 0.2, 0.2);
        let Some(rect) = window_rect(&meta((false, false, false)), &edits, full) else {
            panic!("a small region was not windowed");
        };
        let (x0, y0) = rect.origin;
        let (x1, y1) = (x0 + rect.dims.0, y0 + rect.dims.1);
        if x0 % align != 0 || y0 % align != 0 {
            panic!("origin {x0},{y0} is not aligned to {align}");
        }
        if x0 + margin > 2400 || y0 + margin > 1600 || x1 < 3600 + margin || y1 < 2400 + margin {
            panic!("window {rect:?} does not hold the region plus {margin}");
        }
    }

    #[test]
    fn orientation_moves_the_window_with_the_sensor() {
        let full = (6000, 4000);
        let corner = cropped(0.0, 0.0, 0.1, 0.1);
        let upright = window_rect(&meta((false, false, false)), &corner, full).unwrap();
        let flipped = window_rect(&meta((false, true, true)), &corner, full).unwrap();
        if upright.origin != (0, 0) {
            panic!("the top-left crop did not start at the sensor origin: {upright:?}");
        }
        if flipped.origin.0 + flipped.dims.0 != 6000 || flipped.origin.1 + flipped.dims.1 != 4000 {
            panic!("flips did not move the window to the far corner: {flipped:?}");
        }
        let right = cropped(0.5, 0.0, 0.1, 0.1);
        let turned = window_rect(&meta((true, false, false)), &right, full).unwrap();
        if turned.origin.0 != 0 || turned.origin.1 == 0 {
            panic!("a transposed frame kept the display axes: {turned:?}");
        }
    }

    #[test]
    fn crop_rgb_keeps_the_rows_of_the_rect() {
        let rgb: Vec<u32> = (0..4 * 3 * 3).collect();
        let rect = WindowRect {
            origin: (1, 1),
            dims: (2, 2),
        };
        if crop_rgb(&rgb, 4, rect) != [15, 16, 17, 18, 19, 20, 27, 28, 29, 30, 31, 32] {
            panic!("crop picked the wrong texels");
        }
    }
}
