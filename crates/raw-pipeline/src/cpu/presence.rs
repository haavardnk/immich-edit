use crate::cpu::presence_pyramid::LumaPyramid;
use crate::edits::Edits;
use crate::ops::LinearImage;
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};
use rayon::prelude::*;

pub use crate::presence::has_presence;

pub fn apply_presence(image: &mut LinearImage, edits: &Edits) {
    let amounts = presence_amounts(edits);
    if amounts.is_zero() {
        return;
    }
    let w = image.width as u32;
    let h = image.height as u32;
    let radii = presence_radii(w, h);
    let mips = presence_mips(w, h, radii);
    let levels = presence_pyramid_levels(w, h, radii) as usize;
    let pyramid = LumaPyramid::build(image, levels);
    let img_w = image.width;

    image
        .rgb
        .par_chunks_exact_mut(3)
        .enumerate()
        .for_each(|(i, px)| {
            let x = i % img_w;
            let y = i / img_w;
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let y0 = 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2];
            let y0c = y0.max(1e-5);
            let mut log_gain = 0.0f32;
            if amounts.texture != 0.0 {
                let b = pyramid.sample(mips.texture, fx, fy);
                log_gain += amounts.texture * (y0c / b.max(1e-5)).log2();
            }
            if amounts.clarity != 0.0 {
                let b = pyramid.sample(mips.clarity, fx, fy);
                let mt = smoothstep(0.0, 0.1, y0)
                    * (1.0 - smoothstep(0.9, 1.0, y0))
                    * (1.0 - (2.0 * y0 - 1.0).abs()).max(0.0);
                let ratio = (y0c / b.max(1e-5)).log2();
                let gate = smoothstep(0.015, 0.12, ratio.abs());
                log_gain += amounts.clarity * mt * gate * ratio;
            }
            let mut new_y = y0 * log_gain.exp2();
            if amounts.dehaze != 0.0 {
                let b = pyramid.sample(mips.dehaze, fx, fy);
                new_y += amounts.dehaze * (y0 - b);
            }
            let goal = new_y.max(0.0);
            if y0 <= 1e-5 {
                px[0] = goal;
                px[1] = goal;
                px[2] = goal;
            } else {
                let scale = goal / y0;
                px[0] = (px[0] * scale).max(0.0);
                px[1] = (px[1] * scale).max(0.0);
                px[2] = (px[2] * scale).max(0.0);
            }
        });
}

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
