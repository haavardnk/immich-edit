use crate::edits::{BasicEdits, CropRect, Edits, OutputEdits, ToneEdits};
use crate::frame::{OrientFlips, RawFrame};
use crate::geom::{GeometryTransform, mask_uv_to_display_uv};
use crate::ops::lens_distortion::{distortion_coeffs, distortion_zoom, output_px_to_source_px};
use crate::ops::lens_vignette::{vignette_coeffs, vignette_correction};

const SAMPLE_TARGET: usize = 200_000;
const HIST_BINS: usize = 256;
const MIN_VALID_SAMPLES: u32 = 1000;

fn develop_luma(r: f32, g: f32, b: f32, output: OutputEdits) -> f32 {
    crate::tone::apply_display_luma([r, g, b], output)
}

struct Stats {
    hist: [u32; HIST_BINS],
    total: u32,
    mean_sat: f32,
}

fn hist_percentile(hist: &[u32; HIST_BINS], total: u32, p: f64) -> usize {
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

fn hist_fraction_above(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[threshold..].iter().sum();
    sum as f64 / total.max(1) as f64
}

fn hist_fraction_below(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[..=threshold.min(HIST_BINS - 1)].iter().sum();
    sum as f64 / total.max(1) as f64
}

fn wb_scale(frame: &RawFrame) -> [f32; 3] {
    let wb = frame.wb_coeffs;
    let g = wb[1].max(1e-6);
    let raw_scale = [wb[0] / g, 1.0, wb[2] / g];
    let max_scale = raw_scale[0].max(raw_scale[1]).max(raw_scale[2]);
    let norm = if max_scale > 1.0 {
        1.0 / max_scale
    } else {
        1.0
    };
    [
        raw_scale[0] * norm,
        raw_scale[1] * norm,
        raw_scale[2] * norm,
    ]
}

fn sample_raw_bilinear(frame: &RawFrame, x: f32, y: f32) -> Option<[f32; 3]> {
    let w = frame.width as i32;
    let h = frame.height as i32;
    if w <= 0 || h <= 0 || frame.cpp < 3 {
        return None;
    }
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let tx = x - xi as f32;
    let ty = y - yi as f32;
    let stride = frame.cpp;
    let load = |ix: i32, iy: i32| -> [f32; 3] {
        let cx = ix.clamp(0, w - 1) as usize;
        let cy = iy.clamp(0, h - 1) as usize;
        let off = (cy * frame.width + cx) * stride;
        [frame.data[off], frame.data[off + 1], frame.data[off + 2]]
    };
    let c00 = load(xi, yi);
    let c10 = load(xi + 1, yi);
    let c01 = load(xi, yi + 1);
    let c11 = load(xi + 1, yi + 1);
    let mix = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let a = mix(c00[c], c10[c], tx);
        let b = mix(c01[c], c11[c], tx);
        out[c] = mix(a, b, ty);
    }
    Some(out)
}

fn sensor_to_oriented_uv(px: f32, py: f32, w: usize, h: usize, orient: OrientFlips) -> (f32, f32) {
    let (t, hf, vf) = orient;
    let wf = w as f32;
    let hf32 = h as f32;
    let mut px2 = px;
    let mut py2 = py;
    if hf {
        px2 = wf - px2;
    }
    if vf {
        py2 = hf32 - py2;
    }
    let (ox, oy, ow, oh) = if t {
        (py2, px2, hf32, wf)
    } else {
        (px2, py2, wf, hf32)
    };
    (ox / ow, oy / oh)
}

fn geometry_transform(
    edits: &Edits,
    oriented_w: u32,
    oriented_h: u32,
) -> Option<GeometryTransform> {
    let g = &edits.geometry;
    let crop = g.crop.unwrap_or(CropRect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    });
    let t = GeometryTransform {
        input_w: oriented_w,
        input_h: oriented_h,
        rotate_quarter: g.rotate,
        flip_h: g.flip_h,
        flip_v: g.flip_v,
        angle_deg: g.rotate_angle,
        crop,
        output_w: oriented_w,
        output_h: oriented_h,
    };
    if t.is_identity() { None } else { Some(t) }
}

fn add_sample(
    hist: &mut [u32; HIST_BINS],
    total: &mut u32,
    sat_sum: &mut f64,
    sat_n: &mut u32,
    rgb: [f32; 3],
    output: OutputEdits,
) {
    let [r, gv, b] = rgb;
    let y_srgb = develop_luma(r, gv, b, output);
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

fn collect_stats_direct(frame: &RawFrame, output: OutputEdits) -> Option<Stats> {
    if frame.cpp < 3 {
        return None;
    }
    let pixel_count = frame.data.len() / frame.cpp;
    if pixel_count == 0 {
        return None;
    }
    let step = (pixel_count / SAMPLE_TARGET).max(1);
    let scale = wb_scale(frame);

    let mut hist = [0u32; HIST_BINS];
    let mut sat_sum: f64 = 0.0;
    let mut sat_n: u32 = 0;
    let mut total = 0u32;

    let mut i = 0;
    while i < pixel_count {
        let off = i * frame.cpp;
        let r = (frame.data[off] * scale[0]).max(0.0);
        let gv = (frame.data[off + 1] * scale[1]).max(0.0);
        let b = (frame.data[off + 2] * scale[2]).max(0.0);
        add_sample(
            &mut hist,
            &mut total,
            &mut sat_sum,
            &mut sat_n,
            [r, gv, b],
            output,
        );
        i += step;
    }

    finalize_stats(hist, total, sat_sum, sat_n)
}

fn collect_stats_output(frame: &RawFrame, edits: &Edits) -> Option<Stats> {
    let output = edits.output;
    let w = frame.width;
    let h = frame.height;
    if w == 0 || h == 0 || frame.cpp < 3 {
        return None;
    }
    let scale = wb_scale(frame);

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
        let mut r = (rgb[0] * scale[0]).max(0.0);
        let mut gv = (rgb[1] * scale[1]).max(0.0);
        let mut b = (rgb[2] * scale[2]).max(0.0);

        if vignette_on {
            let dx = px + 0.5 - cx;
            let dy = py + 0.5 - cy;
            let r_norm = (dx * dx + dy * dy).sqrt() * inv_diag;
            let gain = vignette_correction(vk1, vk2, vk3, vig_amount, r_norm).clamp(0.0, 2.5);
            r *= gain;
            gv *= gain;
            b *= gain;
        }

        add_sample(
            &mut hist,
            &mut total,
            &mut sat_sum,
            &mut sat_n,
            [r, gv, b],
            output,
        );
        i += step;
    }

    if total < MIN_VALID_SAMPLES {
        return None;
    }
    finalize_stats(hist, total, sat_sum, sat_n)
}

fn needs_output_pass(edits: &Edits) -> bool {
    edits.lens.distortion_active()
        || edits.lens.vignette_active()
        || edits.geometry.crop.is_some()
        || edits.geometry.rotate != 0
        || edits.geometry.flip_h
        || edits.geometry.flip_v
        || edits.geometry.rotate_angle.abs() > 1e-4
}

pub fn auto_adjust(frame: &RawFrame, context: &Edits) -> Edits {
    let context = context.clamped();
    let stats = if needs_output_pass(&context) {
        collect_stats_output(frame, &context)
            .or_else(|| collect_stats_direct(frame, context.output))
    } else {
        collect_stats_direct(frame, context.output)
    };
    let Some(s) = stats else {
        return Edits::default();
    };

    let p01 = hist_percentile(&s.hist, s.total, 0.01);
    let p50 = hist_percentile(&s.hist, s.total, 0.50);
    let p99 = hist_percentile(&s.hist, s.total, 0.99);

    let range = (p99 as f64 - p01 as f64).max(1.0);

    let highlight_frac = hist_fraction_above(&s.hist, s.total, 240);
    let clipped_frac = hist_fraction_above(&s.hist, s.total, 250);
    let highlight_guard = p99 > 245 || highlight_frac > 0.02 || clipped_frac > 0.005;

    let target_ev = ((128.0 - p50 as f64) * 0.008).clamp(-2.0, 2.0);
    let exposure_ev = if highlight_guard {
        target_ev.min(0.0)
    } else {
        target_ev
    };

    let brightness = if target_ev > 0.0 && exposure_ev < target_ev {
        ((target_ev - exposure_ev) * 70.0).clamp(0.0, 60.0)
    } else {
        0.0
    };

    let mut contrast = 0.0f64;
    if range < 200.0 {
        contrast = ((200.0 / range) - 1.0) * 8.0;
    }
    if highlight_frac > 0.02 {
        contrast *= 0.5;
    }
    contrast = contrast.clamp(-30.0, 30.0);

    let shadow_frac = hist_fraction_below(&s.hist, s.total, 32);
    let mut shadows = 0.0f64;
    if shadow_frac > 0.05 {
        shadows = (shadow_frac * 40.0).min(35.0);
    }

    let mut highlights = 0.0f64;
    if highlight_frac > 0.02 {
        highlights = -(highlight_frac * 120.0).min(70.0);
    }

    let simulated_p01 = (p01 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);
    let simulated_p99 = (p99 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);
    let blacks = -(simulated_p01 * 0.2).clamp(-15.0, 15.0);
    let whites = ((simulated_p99 - 255.0) * 0.15).clamp(-25.0, 0.0);

    let mut vibrance = 0.0f64;
    if s.mean_sat < 0.20 {
        vibrance = (0.20 - s.mean_sat) as f64 * 120.0;
    }
    vibrance = vibrance.clamp(0.0, 40.0);

    Edits {
        basic: BasicEdits {
            exposure_ev,
            brightness,
            contrast,
            vibrance,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights,
            shadows,
            blacks,
            whites,
        },
        color: Default::default(),
        detail: Default::default(),
        effects: Default::default(),
        lens: Default::default(),
        geometry: Default::default(),
        masks: Vec::new(),
        output: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::{GeometryEdits, LensEdits};

    fn make_frame_luma(luma: f32, w: usize, h: usize) -> RawFrame {
        make_frame_with(w, h, |_, _| luma)
    }

    fn make_frame_with<F: Fn(usize, usize) -> f32>(w: usize, h: usize, f: F) -> RawFrame {
        let mut data = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                let v = f(x, y);
                data.push(v);
                data.push(v);
                data.push(v);
            }
        }
        RawFrame {
            width: w,
            height: h,
            cfa_pattern: String::new(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            data,
            cpp: 3,
            orientation: (false, false, false),
            is_raw: false,
            exif: None,
        }
    }

    #[test]
    fn well_exposed_image_small_exposure() {
        let f = make_frame_luma(0.18, 64, 64);
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.exposure_ev.abs() > 0.5 {
            panic!(
                "expected small exposure for well-exposed image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn very_dark_image_gets_positive_exposure() {
        let f = make_frame_luma(0.01, 64, 64);
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.exposure_ev <= 0.3 {
            panic!(
                "expected positive exposure for dark image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn bright_image_gets_negative_or_zero_exposure() {
        let f = make_frame_luma(0.8, 64, 64);
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.exposure_ev > 0.1 {
            panic!(
                "expected non-positive exposure for bright image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn clipped_highlights_get_recovery() {
        let f = make_frame_with(64, 64, |x, y| {
            let i = y * 64 + x;
            if i % 4 == 0 { 0.99 } else { 0.3 }
        });
        let e = auto_adjust(&f, &Edits::default());
        if e.tone.highlights >= 0.0 {
            panic!("expected highlight recovery, got {}", e.tone.highlights);
        }
    }

    #[test]
    fn exposure_capped_with_bright_highlights() {
        let f = make_frame_luma(0.95, 64, 64);
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.exposure_ev > 0.0 {
            panic!(
                "expected non-positive exposure with clipped highlights, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn brightness_lifts_when_highlights_block_exposure() {
        let f = make_frame_with(64, 64, |x, y| {
            let i = y * 64 + x;
            if i % 8 == 0 { 0.99 } else { 0.06 }
        });
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.exposure_ev > 0.0 {
            panic!(
                "expected exposure clamped at 0, got {}",
                e.basic.exposure_ev
            );
        }
        if e.basic.brightness <= 0.0 {
            panic!(
                "expected positive brightness suggestion, got {}",
                e.basic.brightness
            );
        }
    }

    #[test]
    fn lens_vignette_lifts_corner_luma_in_stats() {
        let w = 96usize;
        let h = 96usize;
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let half = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
        let f = make_frame_with(w, h, |x, y| {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let r = (dx * dx + dy * dy).sqrt() / half;
            (0.6 * (1.0 - 0.5 * r * r)).clamp(0.0, 1.0)
        });
        let edits = Edits {
            lens: LensEdits {
                profile_enabled: true,
                vignette_amount: 100.0,
                vk1: -0.5,
                ..Default::default()
            },
            ..Default::default()
        };
        let with_lens = auto_adjust(&f, &edits);
        let without_lens = auto_adjust(&f, &Edits::default());
        if with_lens.basic.exposure_ev > without_lens.basic.exposure_ev + 1e-6 {
            panic!(
                "lens-aware auto should not be more positive; with={} without={}",
                with_lens.basic.exposure_ev, without_lens.basic.exposure_ev
            );
        }
    }

    #[test]
    fn crop_restricts_sampling() {
        let f = make_frame_with(128, 128, |x, _| if x < 64 { 0.02 } else { 0.5 });
        let cropped = Edits {
            geometry: GeometryEdits {
                crop: Some(CropRect {
                    x: 0.5,
                    y: 0.0,
                    w: 0.5,
                    h: 1.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let without = auto_adjust(&f, &Edits::default());
        let with_crop = auto_adjust(&f, &cropped);
        if with_crop.basic.exposure_ev >= without.basic.exposure_ev {
            panic!(
                "expected crop to remove dark half and reduce exposure; with={} without={}",
                with_crop.basic.exposure_ev, without.basic.exposure_ev
            );
        }
    }

    #[test]
    fn auto_does_not_set_unowned_basic_fields() {
        let f = make_frame_luma(0.18, 32, 32);
        let e = auto_adjust(&f, &Edits::default());
        if e.basic.wb_temp != 0.0
            || e.basic.wb_tint != 0.0
            || e.basic.saturation != 0.0
            || e.basic.texture != 0.0
            || e.basic.clarity != 0.0
            || e.basic.dehaze != 0.0
        {
            panic!("auto set non-owned basic fields: {:?}", e.basic);
        }
    }
}
