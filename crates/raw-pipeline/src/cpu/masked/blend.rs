use rayon::prelude::*;

use super::LayerEval;
use super::weight::fold_layer_weight_with_display;
use crate::ops::LinearImage;
use crate::ops::lens_distortion::{LensWarpParams, mask_uv_to_scene_uv};

pub fn blend_layer_images(
    image: &mut LinearImage,
    layer_images: &[LinearImage],
    layers: &[LayerEval],
    lens_warp: &LensWarpParams,
) {
    if layers.is_empty() || layer_images.len() != layers.len() {
        return;
    }
    let w = image.width;
    let inv_w = 1.0 / w.max(1) as f32;
    let inv_h = 1.0 / image.height.max(1) as f32;
    let row_floats = w * 3;
    let warp_active = !lens_warp.is_identity();

    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let row_base = y * row_floats;
            let v = (y as f32 + 0.5) * inv_h;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let u = (x as f32 + 0.5) * inv_w;
                let (su, sv) = if warp_active {
                    let s = mask_uv_to_scene_uv(lens_warp, [u, v]);
                    (s[0], s[1])
                } else {
                    (u, v)
                };
                let display_rgb = crate::tone::apply_rgb([px[0], px[1], px[2]]);
                let o = row_base + x * 3;
                for (li, layer) in layers.iter().enumerate() {
                    let lw = fold_layer_weight_with_display(layer, su, sv, display_rgb);
                    if lw <= 1e-4 {
                        continue;
                    }
                    let src = &layer_images[li].rgb;
                    px[0] += (src[o] - px[0]) * lw;
                    px[1] += (src[o + 1] - px[1]) * lw;
                    px[2] += (src[o + 2] - px[2]) * lw;
                }
            }
        });
}

pub fn build_sharpen_delta_image(
    image: &LinearImage,
    layers: &[LayerEval],
    deltas: &[f32],
    lens_warp: &LensWarpParams,
) -> LinearImage {
    let w = image.width;
    let h = image.height;
    let inv_w = 1.0 / w.max(1) as f32;
    let inv_h = 1.0 / h.max(1) as f32;
    let warp_active = !lens_warp.is_identity();
    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_exact_mut(w * 3)
        .zip(image.rgb.par_chunks_exact(w * 3))
        .enumerate()
        .for_each(|(y, (row, src))| {
            let v = (y as f32 + 0.5) * inv_h;
            for x in 0..w {
                let u = (x as f32 + 0.5) * inv_w;
                let (su, sv) = if warp_active {
                    let s = mask_uv_to_scene_uv(lens_warp, [u, v]);
                    (s[0], s[1])
                } else {
                    (u, v)
                };
                let i = x * 3;
                let display_rgb = crate::tone::apply_rgb([src[i], src[i + 1], src[i + 2]]);
                let mut acc = 0.0;
                for (li, layer) in layers.iter().enumerate() {
                    if deltas[li] == 0.0 {
                        continue;
                    }
                    acc += fold_layer_weight_with_display(layer, su, sv, display_rgb) * deltas[li];
                }
                row[i] = acc;
                row[i + 1] = acc;
                row[i + 2] = acc;
            }
        });
    LinearImage::new(out, w, h)
}

pub fn render_mask_overlay(
    image: &mut LinearImage,
    layer: &LayerEval,
    lens_warp: &LensWarpParams,
    dcp: Option<&crate::ops::ResolvedDcp>,
) {
    let w = image.width;
    let h = image.height;
    let inv_w = 1.0 / w.max(1) as f32;
    let inv_h = 1.0 / h.max(1) as f32;
    let row_floats = w * 3;
    let warp_active = !lens_warp.is_identity();
    let finish = dcp.map(|d| {
        (
            d.look_table.as_deref(),
            d.tone_curve.as_deref().map(Vec::as_slice),
            &d.to_pp,
            &d.from_pp,
        )
    });
    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let v = (y as f32 + 0.5) * inv_h;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let u = (x as f32 + 0.5) * inv_w;
                let (su, sv) = if warp_active {
                    let s = mask_uv_to_scene_uv(lens_warp, [u, v]);
                    (s[0], s[1])
                } else {
                    (u, v)
                };
                let finished = match finish {
                    Some((look, curve, to_pp, from_pp)) => crate::color::apply_dcp_finish(
                        look,
                        curve,
                        to_pp,
                        from_pp,
                        [px[0], px[1], px[2]],
                    ),
                    None => [px[0], px[1], px[2]],
                };
                let display_rgb = crate::tone::apply_rgb(finished);
                let lw = fold_layer_weight_with_display(layer, su, sv, display_rgb);
                let alpha = lw * 0.55;
                px[0] = display_rgb[0] + (1.0 - display_rgb[0]) * alpha;
                px[1] = display_rgb[1] * (1.0 - alpha);
                px[2] = display_rgb[2] * (1.0 - alpha);
            }
        });
}
