use crate::geom::dng_warp::DngWarp;
use crate::ops::LinearImage;
use crate::ops::sample::{sample_channel_bicubic, sample_rgb_bicubic};
use rayon::prelude::*;

pub fn apply_dng_warp(image: &mut LinearImage, warp: &DngWarp) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let frame = warp.frame(w, h);
    let split = warp.is_split();
    let src = &image.rgb;
    let mut out = vec![0.0f32; src.len()];
    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for (x, px) in row.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            if !split {
                let [sx, sy] = frame.source_px(&warp.planes[0], x as f32, y as f32);
                let rgb = sample_rgb_bicubic(src, w, h, sx, sy);
                px.iter_mut().zip(rgb).for_each(|(v, s)| *v = s.max(0.0));
                continue;
            }
            for (c, v) in px.iter_mut().enumerate() {
                let [sx, sy] = frame.source_px(&warp.planes[c], x as f32, y as f32);
                *v = sample_channel_bicubic(src, w, h, sx, sy, c).max(0.0);
            }
        }
    });
    image.rgb = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::dng_warp::WarpPlane;

    fn ramp(w: usize, h: usize) -> LinearImage {
        let rgb = (0..w * h * 3)
            .map(|i| {
                let p = i / 3;
                0.1 + 0.6 * (p % w) as f32 / w as f32 + 0.2 * (p / w) as f32 / h as f32
            })
            .collect();
        LinearImage::new(rgb, w, h)
    }

    #[test]
    fn identity_warp_leaves_pixels_unchanged() {
        let mut image = ramp(33, 21);
        let before = image.rgb.clone();
        let warp = DngWarp {
            planes: [WarpPlane::IDENTITY; 3],
            center: [0.4, 0.6],
        };
        apply_dng_warp(&mut image, &warp);
        let worst = before
            .iter()
            .zip(&image.rgb)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-5, "{worst}");
    }

    #[test]
    fn zooming_warp_pulls_the_edges_inward() {
        let mut image = ramp(64, 32);
        let plane = WarpPlane {
            radial: [0.9, 0.0, 0.0, 0.0],
            tangential: [0.0, 0.0],
        };
        let warp = DngWarp {
            planes: [plane; 3],
            center: [0.5, 0.5],
        };
        let left_before = image.rgb[16 * 64 * 3];
        apply_dng_warp(&mut image, &warp);
        assert!(image.rgb[16 * 64 * 3] > left_before + 0.02);
    }

    #[test]
    fn split_planes_shift_channels_independently() {
        let mut image = ramp(64, 32);
        let mut planes = [WarpPlane::IDENTITY; 3];
        planes[0].radial[0] = 0.95;
        let warp = DngWarp {
            planes,
            center: [0.5, 0.5],
        };
        let before = image.rgb.clone();
        apply_dng_warp(&mut image, &warp);
        let i = (16 * 64 + 2) * 3;
        assert!(image.rgb[i] > before[i] + 0.01);
        assert!((image.rgb[i + 1] - before[i + 1]).abs() < 1e-5);
    }
}
