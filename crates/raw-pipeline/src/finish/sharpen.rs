use rayon::prelude::*;

use super::{FinalImage, FinalPixels};
use crate::frame::{OutputSharpen, SharpenMedia};
use crate::ops::blur::{gaussian_blur, gaussian_kernel};

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
const REFERENCE_PPI: f32 = 300.0;

fn sigma(media: SharpenMedia) -> f32 {
    match media {
        SharpenMedia::Screen => 0.6,
        SharpenMedia::Glossy { ppi } => (ppi as f32 / REFERENCE_PPI * 0.9).clamp(0.5, 4.0),
        SharpenMedia::Matte { ppi } => (ppi as f32 / REFERENCE_PPI * 1.25).clamp(0.5, 4.0),
    }
}

fn strength(sharpen: OutputSharpen) -> f32 {
    let levels = match sharpen.media {
        SharpenMedia::Screen => [0.3, 0.6, 0.9],
        SharpenMedia::Glossy { .. } => [0.5, 0.8, 1.2],
        SharpenMedia::Matte { .. } => [0.7, 1.1, 1.6],
    };
    levels[sharpen.level as usize]
}

pub(super) fn sharpen(image: &mut FinalImage, sharpen: OutputSharpen) {
    let w = image.width as usize;
    let h = image.height as usize;
    if w < 3 || h < 3 {
        return;
    }
    match &mut image.pixels {
        FinalPixels::Rgb8(rgb) => apply(rgb, w, h, sharpen, 255.0, |v| v as u8),
        FinalPixels::Rgb16(rgb) => apply(rgb, w, h, sharpen, 65535.0, |v| v as u16),
    }
}

fn apply<T>(rgb: &mut [T], w: usize, h: usize, sharpen: OutputSharpen, max: f32, from: fn(f32) -> T)
where
    T: Copy + Send + Sync + Into<f32>,
{
    let luma: Vec<f32> = rgb
        .par_chunks(3)
        .map(|p| (LUMA[0] * p[0].into() + LUMA[1] * p[1].into() + LUMA[2] * p[2].into()) / max)
        .collect();
    let blur = gaussian_blur::<1>(&luma, w, h, &gaussian_kernel(sigma(sharpen.media)));
    let k = strength(sharpen);
    let lum = luma.as_slice();
    rgb.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        let ys = [y.saturating_sub(1), y, (y + 1).min(h - 1)];
        for x in 0..w {
            let xs = [x.saturating_sub(1), x, (x + 1).min(w - 1)];
            let (lo, hi) = ys
                .iter()
                .flat_map(|&ny| xs.iter().map(move |&nx| lum[ny * w + nx]))
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                });
            let i = y * w + x;
            let v = lum[i];
            let delta = ((v + k * (v - blur[i])).clamp(lo, hi) - v) * max;
            for px in &mut row[x * 3..x * 3 + 3] {
                *px = from(((*px).into() + delta).clamp(0.0, max).round());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::SharpenLevel;

    fn soft_edge(w: u32, h: u32) -> FinalImage {
        let rgb = (0..w * h)
            .flat_map(|i| {
                let t = ((i % w) as f32 - (w as f32 - 1.0) / 2.0) / 1.5;
                let v = (64.0 + 128.0 / (1.0 + (-t).exp())).round() as u8;
                [v, v, v]
            })
            .collect();
        FinalImage {
            pixels: FinalPixels::Rgb8(rgb),
            width: w,
            height: h,
        }
    }

    fn steepest(image: &FinalImage) -> u8 {
        let FinalPixels::Rgb8(rgb) = &image.pixels else {
            panic!("expected rgb8");
        };
        rgb[32 * 3] - rgb[31 * 3]
    }

    fn sharpened(media: SharpenMedia, level: SharpenLevel) -> FinalImage {
        let mut image = soft_edge(64, 8);
        sharpen(&mut image, OutputSharpen { media, level });
        image
    }

    #[test]
    fn steepens_a_soft_edge_more_at_higher_levels() {
        let base = steepest(&soft_edge(64, 8));
        let [low, standard, high] = [
            SharpenLevel::Low,
            SharpenLevel::Standard,
            SharpenLevel::High,
        ]
        .map(|level| steepest(&sharpened(SharpenMedia::Matte { ppi: 300 }, level)));
        assert!(
            base < low && low < standard && standard <= high,
            "{base} {low} {standard} {high}"
        );
        let mut edge = FinalImage {
            pixels: FinalPixels::Rgb8(
                (0..64 * 4)
                    .flat_map(|i| {
                        let v = [64, 96, 160, 192][(i % 64).clamp(30, 33) as usize - 30];
                        [v, v, v]
                    })
                    .collect(),
            ),
            width: 64,
            height: 4,
        };
        let before = steepest(&edge);
        sharpen(
            &mut edge,
            OutputSharpen {
                media: SharpenMedia::Screen,
                level: SharpenLevel::Standard,
            },
        );
        let screen = steepest(&edge);
        assert!(screen > before, "{before} {screen}");
    }

    #[test]
    fn never_overshoots_the_neighbourhood_or_changes_flat_areas() {
        let image = sharpened(SharpenMedia::Matte { ppi: 300 }, SharpenLevel::High);
        let FinalPixels::Rgb8(rgb) = &image.pixels else {
            panic!("expected rgb8");
        };
        assert!(rgb.iter().all(|&v| (64..=192).contains(&v)));
        assert_eq!(&rgb[..3], &[64, 64, 64]);
        assert!(rgb.chunks_exact(3).all(|p| p[0] == p[1] && p[1] == p[2]));
    }

    #[test]
    fn sharpens_sixteen_bit_output() {
        let FinalPixels::Rgb8(edge) = soft_edge(64, 4).pixels else {
            panic!("expected rgb8");
        };
        let original: Vec<u16> = edge.iter().map(|&v| u16::from(v) * 257).collect();
        let mut image = FinalImage {
            pixels: FinalPixels::Rgb16(original.clone()),
            width: 64,
            height: 4,
        };
        sharpen(
            &mut image,
            OutputSharpen {
                media: SharpenMedia::Glossy { ppi: 300 },
                level: SharpenLevel::Standard,
            },
        );
        let FinalPixels::Rgb16(rgb) = &image.pixels else {
            panic!("expected rgb16");
        };
        let slope = |px: &[u16]| px[33 * 3].abs_diff(px[30 * 3]);
        assert!(slope(rgb) > slope(&original));
    }

    #[test]
    fn print_radius_follows_pixel_density() {
        assert!(
            sigma(SharpenMedia::Glossy { ppi: 600 }) > sigma(SharpenMedia::Glossy { ppi: 300 })
        );
        assert!(sigma(SharpenMedia::Matte { ppi: 300 }) > sigma(SharpenMedia::Glossy { ppi: 300 }));
        assert_eq!(sigma(SharpenMedia::Matte { ppi: 72 }), 0.5);
        assert_eq!(sigma(SharpenMedia::Glossy { ppi: 4000 }), 4.0);
    }
}
