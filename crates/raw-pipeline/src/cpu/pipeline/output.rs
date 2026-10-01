use multiversion::multiversion;
use rayon::prelude::*;

use crate::edits::Edits;
use crate::frame::{OutputColorSpace, RenderOptions};
use crate::histogram::{self, Bins, Histogram};
use crate::ops::curves::{CurveLuts, apply_display_curves};

pub(super) type DcpFinish<'a> = (
    Option<&'a crate::dcp::HueSatMap>,
    Option<&'a crate::dcp::ToneCurve>,
    &'a [[f32; 3]; 3],
    &'a [[f32; 3]; 3],
);

pub(super) struct Histograms {
    pub display: Histogram,
    pub linear: Histogram,
}

#[derive(Clone, Copy, Default)]
pub(super) struct FinishOptions<'a> {
    pub want_16bit: bool,
    pub display_ready: bool,
    pub lut: Option<(&'a crate::lut::CubeLut, f32)>,
    pub curves: Option<&'a CurveLuts>,
    pub dcp_finish: Option<DcpFinish<'a>>,
    pub color_space: OutputColorSpace,
    pub gamut_warn: bool,
    pub clip_warn: bool,
    pub histogram: bool,
}

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

pub(super) fn resolve_lut(
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<Option<(std::sync::Arc<crate::lut::CubeLut>, f32)>> {
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

pub(super) fn finish_output(
    linear: Vec<f32>,
    w: usize,
    h: usize,
    opts: FinishOptions,
) -> (Vec<u8>, Option<Vec<u16>>, Option<Histograms>) {
    let _span = tracing::debug_span!("cpu.finish_output_histogram", w = w, h = h).entered();
    let want_16bit = opts.want_16bit;
    let pixel_count = w * h;
    let n = linear.len();
    let mut rgb_u8 = vec![0u8; n];
    let mut rgb_u16: Vec<u16> = if want_16bit {
        vec![0u16; n]
    } else {
        Vec::new()
    };
    let step = histogram::sample_step(pixel_count);
    let chunk_px = histogram::chunk_pixels(pixel_count);
    let chunk = chunk_px * 3;
    let finish = Finish { w, step, opts };

    let zero_bins = || (Bins::zero(), Bins::zero());
    let merge_bins = |a: (Bins, Bins), b: (Bins, Bins)| (a.0.merge(b.0), a.1.merge(b.1));
    let (lin_bins, dis_bins) = if want_16bit {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .zip(rgb_u16.par_chunks_mut(chunk))
            .fold(zero_bins, |mut acc, (((ci, s), u8c), u16c)| {
                finish_chunk(&finish, ci * chunk_px, s, u8c, Some(u16c), &mut acc);
                acc
            })
            .reduce(zero_bins, merge_bins)
    } else {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .fold(zero_bins, |mut acc, ((ci, s), u8c)| {
                finish_chunk(&finish, ci * chunk_px, s, u8c, None, &mut acc);
                acc
            })
            .reduce(zero_bins, merge_bins)
    };

    let rgb_u16 = if want_16bit { Some(rgb_u16) } else { None };
    let histograms = opts.histogram.then(|| Histograms {
        display: dis_bins.into_histogram(),
        linear: lin_bins.into_histogram(),
    });
    (rgb_u8, rgb_u16, histograms)
}

struct Finish<'a> {
    w: usize,
    step: usize,
    opts: FinishOptions<'a>,
}

impl Finish<'_> {
    #[inline(always)]
    fn finalize(&self, lr: f32, lg: f32, lb: f32) -> ([f32; 3], bool) {
        let opts = &self.opts;
        if opts.display_ready {
            return ([lr, lg, lb], false);
        }
        let finished = match opts.dcp_finish {
            Some((look, curve, to_pp, from_pp)) => {
                crate::color::apply_dcp_finish(look, curve, to_pp, from_pp, [lr, lg, lb])
            }
            None => [lr, lg, lb],
        };
        let clip = opts.gamut_warn && crate::tone::is_out_of_gamut(finished, opts.color_space);
        let display = crate::tone::apply_rgb_cs(finished, opts.color_space);
        let display = match opts.curves {
            Some(luts) => apply_display_curves(luts, display),
            None => display,
        };
        let graded = opts.lut.map_or(display, |(lut, amount)| {
            lut.apply(display, amount, opts.color_space)
        });
        (graded, clip)
    }
}

#[multiversion(targets("x86_64+avx2", "x86_64+sse4.2"))]
fn finish_chunk(
    f: &Finish,
    base_px: usize,
    s: &[f32],
    u8c: &mut [u8],
    mut u16c: Option<&mut [u16]>,
    acc: &mut (Bins, Bins),
) {
    for (p, (px_in, px_out)) in s.chunks_exact(3).zip(u8c.chunks_exact_mut(3)).enumerate() {
        let i = p * 3;
        let lr = px_in[0];
        let lg = px_in[1];
        let lb = px_in[2];
        let ([tr, tg, tb], clip) = f.finalize(lr, lg, lb);
        let abs_px = base_px + p;
        let px = (abs_px % f.w) as u32;
        let py = (abs_px / f.w) as u32;
        let ru = quantize_u8_dithered(tr, px, py, 0);
        let gu = quantize_u8_dithered(tg, px, py, 1);
        let bu = quantize_u8_dithered(tb, px, py, 2);
        px_out[0] = ru;
        px_out[1] = gu;
        px_out[2] = bu;
        if let Some(dst) = u16c.as_deref_mut() {
            dst[i] = (tr.clamp(0.0, 1.0) * 65535.0).round() as u16;
            dst[i + 1] = (tg.clamp(0.0, 1.0) * 65535.0).round() as u16;
            dst[i + 2] = (tb.clamp(0.0, 1.0) * 65535.0).round() as u16;
        }
        if f.opts.histogram && abs_px % f.step == 0 {
            acc.0.add_linear(lr, lg, lb);
            acc.1.add_display(ru, gu, bu);
        }
        if let Some(paint) = crate::warn::classify([tr, tg, tb], clip, f.opts.clip_warn) {
            px_out.copy_from_slice(&paint);
        }
    }
}

#[cfg(test)]
mod tests;
