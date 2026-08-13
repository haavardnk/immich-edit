use rayon::prelude::*;

use super::bins::{HistBins, bins_to_histogram, fold_display, fold_linear, merge_bins, zero_bins};
use crate::edits::Edits;
use crate::frame::{OutputColorSpace, RenderOptions};
use crate::histogram::{self, Histogram};

pub(super) type DcpFinish<'a> = (
    Option<&'a crate::dcp::HueSatMap>,
    Option<&'a [[f32; 2]]>,
    &'a [[f32; 3]; 3],
    &'a [[f32; 3]; 3],
);

#[inline(always)]
fn dither_hash(x: u32, y: u32, c: u32) -> f32 {
    let mut h =
        x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ c.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

#[inline(always)]
fn quantize_u8_dithered(v: f32, x: u32, y: u32, c: u32) -> u8 {
    let tpdf = dither_hash(x, y, c * 2) - dither_hash(x, y, c * 2 + 1);
    ((v.clamp(0.0, 1.0) * 255.0 + tpdf).round()).clamp(0.0, 255.0) as u8
}

#[inline]
fn apply_display_lut(rgb: [f32; 3], lut: Option<(&crate::lut::Lut3d, f32)>) -> [f32; 3] {
    match lut {
        Some((l, amount)) => {
            let sampled = l.sample(rgb);
            [
                rgb[0] + amount * (sampled[0] - rgb[0]),
                rgb[1] + amount * (sampled[1] - rgb[1]),
                rgb[2] + amount * (sampled[2] - rgb[2]),
            ]
        }
        None => rgb,
    }
}

pub(super) fn resolve_lut(
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<Option<(std::sync::Arc<crate::lut::Lut3d>, f32)>> {
    if !edits.color.lut_3d.is_active() {
        return Ok(None);
    }
    let id = edits
        .color
        .lut_3d
        .lut_id
        .as_ref()
        .ok_or_else(|| crate::PipelineError::Render("lut id missing".into()))?;
    let lut = options
        .luts
        .get(id)
        .ok_or_else(|| crate::PipelineError::Render(format!("lut {id} not loaded")))?;
    Ok(Some((
        lut.clone(),
        (edits.color.lut_3d.amount / 100.0) as f32,
    )))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_output(
    linear: Vec<f32>,
    w: usize,
    h: usize,
    want_16bit: bool,
    display_ready: bool,
    lut: Option<(&crate::lut::Lut3d, f32)>,
    dcp_finish: Option<DcpFinish>,
    color_space: OutputColorSpace,
    gamut_warn: bool,
    clip_warn: bool,
) -> (Vec<u8>, Option<Vec<u16>>, Histogram, Histogram) {
    let _span = tracing::debug_span!("cpu.finish_output_histogram", w = w, h = h).entered();
    let pixel_count = w * h;
    let n = linear.len();
    let mut rgb_u8 = vec![0u8; n];
    let mut rgb_u16: Vec<u16> = if want_16bit {
        vec![0u16; n]
    } else {
        Vec::new()
    };
    let step = if pixel_count > 500_000 { 2 } else { 1 };
    let chunk_px = histogram::chunk_pixels(pixel_count);
    let chunk = chunk_px * 3;

    let finalize = |lr: f32, lg: f32, lb: f32| -> ([f32; 3], bool) {
        if display_ready {
            return ([lr, lg, lb], false);
        }
        let finished = match dcp_finish {
            Some((look, curve, to_pp, from_pp)) => {
                crate::color::apply_dcp_finish(look, curve, to_pp, from_pp, [lr, lg, lb])
            }
            None => [lr, lg, lb],
        };
        let clip = gamut_warn && crate::tone::is_out_of_gamut(finished, color_space);
        let display = crate::tone::apply_rgb_cs(finished, color_space);
        (apply_display_lut(display, lut), clip)
    };

    let process = |base_px: usize,
                   s: &[f32],
                   u8c: &mut [u8],
                   mut u16c: Option<&mut [u16]>,
                   acc: &mut (HistBins, HistBins)| {
        let mut i = 0;
        let mut p = 0usize;
        while i + 2 < s.len() {
            let lr = s[i];
            let lg = s[i + 1];
            let lb = s[i + 2];
            let ([tr, tg, tb], clip) = finalize(lr, lg, lb);
            let abs_px = base_px + p;
            let px = (abs_px % w) as u32;
            let py = (abs_px / w) as u32;
            let ru = quantize_u8_dithered(tr, px, py, 0);
            let gu = quantize_u8_dithered(tg, px, py, 1);
            let bu = quantize_u8_dithered(tb, px, py, 2);
            u8c[i] = ru;
            u8c[i + 1] = gu;
            u8c[i + 2] = bu;
            if let Some(dst) = u16c.as_deref_mut() {
                dst[i] = (tr.clamp(0.0, 1.0) * 65535.0) as u16;
                dst[i + 1] = (tg.clamp(0.0, 1.0) * 65535.0) as u16;
                dst[i + 2] = (tb.clamp(0.0, 1.0) * 65535.0) as u16;
            }
            if p % step == 0 {
                fold_linear(&mut acc.0, lr, lg, lb);
                fold_display(&mut acc.1, ru, gu, bu);
            }
            if let Some(paint) = crate::warn::classify([tr, tg, tb], clip, clip_warn) {
                u8c[i] = paint[0];
                u8c[i + 1] = paint[1];
                u8c[i + 2] = paint[2];
            }
            i += 3;
            p += 1;
        }
    };

    let (lin_bins, dis_bins) = if want_16bit {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .zip(rgb_u16.par_chunks_mut(chunk))
            .fold(zero_bins, |mut acc, (((ci, s), u8c), u16c)| {
                process(ci * chunk_px, s, u8c, Some(u16c), &mut acc);
                acc
            })
            .reduce(zero_bins, |a, b| {
                (merge_bins(a.0, b.0), merge_bins(a.1, b.1))
            })
    } else {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .fold(zero_bins, |mut acc, ((ci, s), u8c)| {
                process(ci * chunk_px, s, u8c, None, &mut acc);
                acc
            })
            .reduce(zero_bins, |a, b| {
                (merge_bins(a.0, b.0), merge_bins(a.1, b.1))
            })
    };

    let rgb_u16 = if want_16bit { Some(rgb_u16) } else { None };
    (
        rgb_u8,
        rgb_u16,
        bins_to_histogram(dis_bins),
        bins_to_histogram(lin_bins),
    )
}

#[cfg(test)]
mod tests;
