use super::sample::{
    develop_luma, display_rgb, geometry_transform, sample_raw_bilinear, sensor_to_oriented_uv,
};
use super::{HIST_BINS, MIN_VALID_SAMPLES, SAMPLE_TARGET};
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::geom::mask_uv_to_display_uv;
use crate::ops::lens_distortion::{distortion_coeffs, distortion_zoom, output_px_to_source_px};
use crate::ops::lens_vignette::{vignette_coeffs, vignette_correction};

pub(super) struct Stats {
    pub(super) hist: [u32; HIST_BINS],
    pub(super) total: u32,
    pub(super) mean_sat: f32,
}

pub(super) fn hist_percentile(hist: &[u32; HIST_BINS], total: u32, p: f64) -> usize {
    let target = (total as f64 * p) as u32;
    let mut cumulative = 0u32;
    for (i, &v) in hist.iter().enumerate() {
        cumulative += v;
        if cumulative >= target {
            return i;
        }
    }
    HIST_BINS - 1
}

pub(super) fn hist_fraction_above(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[threshold..].iter().sum();
    sum as f64 / total.max(1) as f64
}

pub(super) fn hist_fraction_below(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[..=threshold.min(HIST_BINS - 1)].iter().sum();
    sum as f64 / total.max(1) as f64
}
fn add_sample(
    hist: &mut [u32; HIST_BINS],
    total: &mut u32,
    sat_sum: &mut f64,
    sat_n: &mut u32,
    rgb: [f32; 3],
) {
    let [r, gv, b] = rgb;
    let y_srgb = develop_luma(r, gv, b);
    let bin = (y_srgb * 255.0).round().clamp(0.0, 255.0) as usize;
    hist[bin] += 1;
    *total += 1;
    if y_srgb > 0.05 && y_srgb < 0.95 {
        let mx = r.max(gv).max(b);
        let mn = r.min(gv).min(b);
        if mx > 1e-4 {
            *sat_sum += ((mx - mn) / mx) as f64;
            *sat_n += 1;
        }
    }
}

fn finalize_stats(hist: [u32; HIST_BINS], total: u32, sat_sum: f64, sat_n: u32) -> Option<Stats> {
    if total == 0 {
        return None;
    }
    let mean_sat = if sat_n > 0 {
        (sat_sum / sat_n as f64) as f32
    } else {
        0.0
    };
    Some(Stats {
        hist,
        total,
        mean_sat,
    })
}
pub(super) fn collect_stats_direct(
    frame: &RawFrame,
    wb: [f32; 3],
    m: [[f32; 3]; 3],
) -> Option<Stats> {
    if frame.cpp < 3 {
        return None;
    }
    let pixel_count = frame.data.len() / frame.cpp;
    if pixel_count == 0 {
        return None;
    }
    let step = (pixel_count / SAMPLE_TARGET).max(1);

    let mut hist = [0u32; HIST_BINS];
    let mut sat_sum: f64 = 0.0;
    let mut sat_n: u32 = 0;
    let mut total = 0u32;

    let mut i = 0;
    while i < pixel_count {
        let off = i * frame.cpp;
        let rgb = display_rgb(
            [frame.data[off], frame.data[off + 1], frame.data[off + 2]],
            wb,
            m,
        );
        add_sample(&mut hist, &mut total, &mut sat_sum, &mut sat_n, rgb);
        i += step;
    }

    finalize_stats(hist, total, sat_sum, sat_n)
}

pub(super) fn collect_stats_output(
    frame: &RawFrame,
    edits: &Edits,
    wb: [f32; 3],
    m: [[f32; 3]; 3],
) -> Option<Stats> {
    let w = frame.width;
    let h = frame.height;
    if w == 0 || h == 0 || frame.cpp < 3 {
        return None;
    }

    let (orient_t, _, _) = frame.orientation;
    let (oriented_w, oriented_h) = if orient_t { (h, w) } else { (w, h) };
    let geom = geometry_transform(edits, oriented_w as u32, oriented_h as u32);

    let lens = &edits.lens;
    let (k1, k2, k3) = distortion_coeffs(lens);
    let zoom = distortion_zoom(lens);
    let distortion_on = lens.distortion_active();
    let (vk1, vk2, vk3, vig_amount) = vignette_coeffs(lens);
    let vignette_on = lens.vignette_active();
    let constrain = lens.constrain_crop;

    let total_pixels = w * h;
    let step = (total_pixels / SAMPLE_TARGET).max(1);
    let half_diag = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
    let inv_diag = 1.0 / half_diag;
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let mut hist = [0u32; HIST_BINS];
    let mut sat_sum: f64 = 0.0;
    let mut sat_n: u32 = 0;
    let mut total = 0u32;

    let mut i = 0;
    while i < total_pixels {
        let px = (i % w) as f32;
        let py = (i / w) as f32;

        let (sx, sy) = if distortion_on {
            output_px_to_source_px(k1, k2, k3, zoom, w as u32, h as u32, px, py)
        } else {
            (px, py)
        };

        if distortion_on
            && !constrain
            && (sx < 0.0 || sy < 0.0 || sx > (w as f32) - 1.0 || sy > (h as f32) - 1.0)
        {
            i += step;
            continue;
        }

        if let Some(geom) = &geom {
            let (u, v) = sensor_to_oriented_uv(px + 0.5, py + 0.5, w, h, frame.orientation);
            let d = mask_uv_to_display_uv(geom, [u, v]);
            if d[0] < 0.0 || d[0] > 1.0 || d[1] < 0.0 || d[1] > 1.0 {
                i += step;
                continue;
            }
        }

        let Some(rgb) = sample_raw_bilinear(frame, sx, sy) else {
            i += step;
            continue;
        };
        let mut cam = [rgb[0].max(0.0), rgb[1].max(0.0), rgb[2].max(0.0)];

        if vignette_on {
            let dx = px + 0.5 - cx;
            let dy = py + 0.5 - cy;
            let r_norm = (dx * dx + dy * dy).sqrt() * inv_diag;
            let gain = vignette_correction(vk1, vk2, vk3, vig_amount, r_norm);
            cam[0] *= gain;
            cam[1] *= gain;
            cam[2] *= gain;
        }

        add_sample(
            &mut hist,
            &mut total,
            &mut sat_sum,
            &mut sat_n,
            display_rgb(cam, wb, m),
        );
        i += step;
    }

    if total < MIN_VALID_SAMPLES {
        return None;
    }
    finalize_stats(hist, total, sat_sum, sat_n)
}

pub(super) fn needs_output_pass(edits: &Edits) -> bool {
    edits.lens.distortion_active()
        || edits.lens.vignette_active()
        || edits.geometry.crop.is_some()
        || edits.geometry.rotate != 0
        || edits.geometry.flip_h
        || edits.geometry.flip_v
        || edits.geometry.rotate_angle.abs() > 1e-4
}
