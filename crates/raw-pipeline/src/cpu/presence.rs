use crate::cpu::presence_pyramid::LumaPyramid;
use crate::edits::Edits;
use crate::math::{luma, smoothstep};
use crate::ops::LinearImage;
use crate::ops::presence::{clarity_midtones, presence_amounts, presence_blurs};
use rayon::prelude::*;

pub use crate::ops::presence::has_presence;

pub fn apply_presence(image: &mut LinearImage, edits: &Edits) {
    let amounts = presence_amounts(edits);
    if amounts.is_zero() {
        return;
    }
    let blurs = presence_blurs(image.width as u32, image.height as u32);
    let pyramid = LumaPyramid::build(image, blurs.levels() as usize);
    let texture = (amounts.texture != 0.0).then(|| pyramid.base(blurs.texture));
    let clarity = (amounts.clarity != 0.0).then(|| pyramid.base(blurs.clarity));
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
            let y0 = luma(px[0], px[1], px[2]);
            let y0c = y0.max(1e-5);
            let mut log_gain = 0.0f32;
            if let Some(base) = &texture {
                let b = base.sample(fx, fy);
                log_gain += amounts.texture * (y0c / b.max(1e-5)).log2();
            }
            if let Some(base) = &clarity {
                let b = base.sample(fx, fy);
                let mt = clarity_midtones(y0, amounts.exposure);
                let ratio = (y0c / b.max(1e-5)).log2();
                let gate = smoothstep(0.015, 0.12, ratio.abs());
                log_gain += amounts.clarity * mt * gate * ratio;
            }
            let new_y = y0 * log_gain.exp2();
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
