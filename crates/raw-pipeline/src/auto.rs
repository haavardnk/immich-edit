use crate::edits::{BasicEdits, CropRect, Edits, ToneEdits};
use crate::frame::{OrientFlips, RawFrame};
use crate::geom::{GeometryTransform, mask_uv_to_display_uv};
use crate::ops::lens_distortion::{distortion_coeffs, distortion_zoom, output_px_to_source_px};
use crate::ops::lens_vignette::{vignette_coeffs, vignette_correction};

const SAMPLE_TARGET: usize = 200_000;
const HIST_BINS: usize = 256;
const MIN_VALID_SAMPLES: u32 = 1000;

fn camera_wb_coeffs(raw: [f32; 4]) -> [f32; 3] {
    if raw[0] == 0.0 && raw[1] == 0.0 && raw[2] == 0.0 {
        return [1.0, 1.0, 1.0];
    }
    if raw[1] > 0.0 {
        return [raw[0] / raw[1], 1.0, raw[2] / raw[1]];
    }
    [raw[0], raw[1], raw[2]]
}

fn parse_cfa(cfa_pattern: &str) -> [u8; 4] {
    let mut cfa = *b"RGGB";
    for (i, b) in cfa_pattern.bytes().take(4).enumerate() {
        cfa[i] = b;
    }
    cfa
}

fn cfa_block(cfa_pattern: &str) -> (usize, Vec<usize>) {
    let to_channel = |b: u8| match b {
        b'R' => 0usize,
        b'B' => 2,
        _ => 1,
    };
    match crate::cpu::demosaic::parse_xtrans(cfa_pattern) {
        Some(pattern) => (6, pattern.iter().map(|b| to_channel(*b)).collect()),
        None => (
            2,
            parse_cfa(cfa_pattern)
                .iter()
                .map(|b| to_channel(*b))
                .collect(),
        ),
    }
}

fn mosaic_block_rgb(
    data: &[f32],
    w: usize,
    dim: usize,
    table: &[usize],
    bx: usize,
    by: usize,
) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    let mut counts = [0.0f32; 3];
    for (dy, dx) in (0..dim).flat_map(|dy| (0..dim).map(move |dx| (dy, dx))) {
        let ch = table[dy * dim + dx];
        acc[ch] += data[(by + dy) * w + bx + dx];
        counts[ch] += 1.0;
    }
    for ch in 0..3 {
        if counts[ch] > 0.0 {
            acc[ch] /= counts[ch];
        }
    }
    acc
}

pub fn scale_matrix(m: [[f32; 3]; 3], gain: f32) -> [[f32; 3]; 3] {
    [
        [m[0][0] * gain, m[0][1] * gain, m[0][2] * gain],
        [m[1][0] * gain, m[1][1] * gain, m[1][2] * gain],
        [m[2][0] * gain, m[2][1] * gain, m[2][2] * gain],
    ]
}

fn display_color(frame: &RawFrame) -> ([f32; 3], [[f32; 3]; 3]) {
    let wb = camera_wb_coeffs(frame.wb_coeffs);
    let xyz_to_cam =
        crate::color::resolve_xyz_to_cam(&frame.color_matrices, frame.wb_coeffs, frame.xyz_to_cam);
    if !frame.is_raw || crate::color::is_unusable_matrix(&xyz_to_cam) {
        return (wb, crate::color::identity_3x3());
    }
    (wb, crate::color::cam_to_srgb_matrix(xyz_to_cam))
}

fn display_rgb(raw: [f32; 3], wb: [f32; 3], m: [[f32; 3]; 3]) -> [f32; 3] {
    let r = (raw[0] * wb[0]).max(0.0);
    let g = (raw[1] * wb[1]).max(0.0);
    let b = (raw[2] * wb[2]).max(0.0);
    [
        (m[0][0] * r + m[0][1] * g + m[0][2] * b).max(0.0),
        (m[1][0] * r + m[1][1] * g + m[1][2] * b).max(0.0),
        (m[2][0] * r + m[2][1] * g + m[2][2] * b).max(0.0),
    ]
}

fn develop_luma(r: f32, g: f32, b: f32) -> f32 {
    crate::tone::apply_display_luma([r, g, b])
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
        perspective_forward: g.perspective_forward(),
        perspective_inverse: g.perspective_inverse(),
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

fn decimate_mosaic(frame: &RawFrame) -> Option<RawFrame> {
    if frame.cpp != 1 || frame.cfa_pattern.is_empty() {
        return None;
    }
    let (dim, table) = cfa_block(&frame.cfa_pattern);
    let bw = frame.width / dim;
    let bh = frame.height / dim;
    if bw == 0 || bh == 0 {
        return None;
    }
    let stride = ((bw * bh) / SAMPLE_TARGET).isqrt().clamp(1, bw.min(bh));
    let out_w = bw / stride;
    let out_h = bh / stride;
    let mut data = Vec::with_capacity(out_w * out_h * 3);
    for (oy, ox) in (0..out_h).flat_map(|oy| (0..out_w).map(move |ox| (oy, ox))) {
        let rgb = mosaic_block_rgb(
            &frame.data,
            frame.width,
            dim,
            &table,
            ox * stride * dim,
            oy * stride * dim,
        );
        data.extend_from_slice(&rgb);
    }
    Some(RawFrame {
        width: out_w,
        height: out_h,
        cfa_pattern: String::new(),
        bps: frame.bps,
        wb_coeffs: frame.wb_coeffs,
        xyz_to_cam: frame.xyz_to_cam,
        color_matrices: frame.color_matrices.clone(),
        data,
        cpp: 3,
        orientation: frame.orientation,
        is_raw: frame.is_raw,
        capture_sigma: None,
        model: frame.model.clone(),
        exif: None,
    })
}

fn collect_stats_direct(frame: &RawFrame, wb: [f32; 3], m: [[f32; 3]; 3]) -> Option<Stats> {
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

fn collect_stats_output(
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
            let gain = vignette_correction(vk1, vk2, vk3, vig_amount, r_norm).clamp(0.0, 2.5);
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
    let (wb, m) = display_color(frame);
    let decimated = decimate_mosaic(frame);
    let frame = decimated.as_ref().unwrap_or(frame);
    let stats = if needs_output_pass(&context) {
        collect_stats_output(frame, &context, wb, m).or_else(|| collect_stats_direct(frame, wb, m))
    } else {
        collect_stats_direct(frame, wb, m)
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

    let target_ev = ((128.0 - p50 as f64) * 0.008).clamp(-2.0, 2.0);
    let ev_headroom = ((252.0 - p99 as f64) / 20.0).max(0.0);
    let exposure_ev = if target_ev > 0.0 {
        let cap = if clipped_frac > 0.02 {
            0.0
        } else {
            ev_headroom.max(0.2)
        };
        target_ev.min(cap)
    } else {
        target_ev
    };

    let brightness = ((target_ev - exposure_ev) * 45.0).clamp(0.0, 30.0);

    let mut contrast = 8.0f64;
    if range < 200.0 {
        contrast += ((200.0 / range) - 1.0) * 8.0;
    }
    if highlight_frac > 0.05 {
        contrast *= 0.6;
    }
    contrast = contrast.clamp(-30.0, 30.0);

    let shadow_frac = hist_fraction_below(&s.hist, s.total, 32);
    let mut shadows = 0.0f64;
    if shadow_frac > 0.05 {
        shadows = (shadow_frac * 40.0).min(35.0);
    }

    let simulated_p01 = (p01 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);
    let simulated_p99 = (p99 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);

    let mut highlights = 0.0f64;
    if highlight_frac > 0.01 {
        highlights = -(highlight_frac * 150.0).min(60.0);
    }
    if clipped_frac > 0.005 {
        highlights -= (clipped_frac * 600.0).min(25.0);
    }
    if simulated_p99 > 245.0 {
        highlights -= (simulated_p99 - 245.0) * 1.2;
    }
    highlights = highlights.clamp(-70.0, 0.0);

    let mut clarity = 7.0f64;
    if highlight_frac > 0.05 {
        clarity *= 0.5;
    }
    if shadow_frac > 0.25 {
        clarity *= 0.6;
    }
    clarity = clarity.clamp(0.0, 12.0);

    let blacks = -(simulated_p01 * 0.2).clamp(-15.0, 15.0);
    let whites = ((simulated_p99 - 255.0) * 0.15).clamp(-25.0, 0.0);

    let vibrance = (5.0 + (0.34 - s.mean_sat as f64) * 55.0).clamp(3.0, 28.0);

    Edits {
        basic: BasicEdits {
            exposure_ev,
            brightness,
            contrast,
            clarity,
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
        retouch: Vec::new(),
        unknown_ops: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::{GeometryEdits, LensEdits};

    fn make_frame_luma(luma: f32, w: usize, h: usize) -> RawFrame {
        make_frame_with(w, h, |_, _| luma)
    }

    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

    fn make_mosaic_frame_with<F: Fn(usize, usize) -> f32>(
        w: usize,
        h: usize,
        cfa: &str,
        f: F,
    ) -> RawFrame {
        let mut data = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                data.push(f(x, y));
            }
        }
        RawFrame {
            width: w,
            height: h,
            cfa_pattern: cfa.to_string(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            data,
            cpp: 1,
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
            exif: None,
        }
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
            capture_sigma: None,
            model: String::new(),
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
    fn blown_highlights_trigger_recovery() {
        let f = make_frame_with(64, 64, |x, y| {
            let i = y * 64 + x;
            if i % 8 == 0 { 0.99 } else { 0.06 }
        });
        let e = auto_adjust(&f, &Edits::default());
        if e.tone.highlights >= 0.0 {
            panic!("expected highlight recovery, got {}", e.tone.highlights);
        }
        if e.basic.exposure_ev + e.basic.brightness <= 0.0 {
            panic!(
                "expected some lift for dark scene, ev={} bri={}",
                e.basic.exposure_ev, e.basic.brightness
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
    fn mosaic_auto_matches_demosaiced_auto() {
        let scene = |x: usize, y: usize| {
            let u = x as f32 / 384.0;
            let v = y as f32 / 384.0;
            (0.02 + 0.55 * v + 0.15 * u).clamp(0.0, 1.0)
        };
        let cropped = Edits {
            geometry: GeometryEdits {
                crop: Some(CropRect {
                    x: 0.25,
                    y: 0.25,
                    w: 0.5,
                    h: 0.5,
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let rgb = make_frame_with(384, 384, scene);
        for cfa in ["RGGB", XTRANS] {
            let mosaic = make_mosaic_frame_with(384, 384, cfa, scene);
            for context in [Edits::default(), cropped.clone()] {
                let want = auto_adjust(&rgb, &context);
                let got = auto_adjust(&mosaic, &context);
                if got == Edits::default() {
                    panic!("auto produced no edits for cfa '{cfa}'");
                }
                if (got.basic.exposure_ev - want.basic.exposure_ev).abs() > 0.1 {
                    panic!(
                        "cfa '{cfa}': mosaic exposure {} disagrees with demosaiced {}",
                        got.basic.exposure_ev, want.basic.exposure_ev
                    );
                }
                if (got.basic.contrast - want.basic.contrast).abs() > 1.0 {
                    panic!(
                        "cfa '{cfa}': mosaic contrast {} disagrees with demosaiced {}",
                        got.basic.contrast, want.basic.contrast
                    );
                }
            }
        }
    }

    #[test]
    fn mosaic_auto_reads_scene_brightness() {
        for cfa in ["RGGB", XTRANS] {
            let dark = make_mosaic_frame_with(384, 384, cfa, |_, _| 0.01);
            let bright = make_mosaic_frame_with(384, 384, cfa, |_, _| 0.8);
            let e_dark = auto_adjust(&dark, &Edits::default());
            let e_bright = auto_adjust(&bright, &Edits::default());
            if e_dark.basic.exposure_ev <= 0.0 {
                panic!(
                    "cfa '{cfa}': dark mosaic should lift exposure, got {}",
                    e_dark.basic.exposure_ev
                );
            }
            if e_bright.basic.exposure_ev >= e_dark.basic.exposure_ev {
                panic!(
                    "cfa '{cfa}': bright mosaic should need less exposure than dark; bright={} dark={}",
                    e_bright.basic.exposure_ev, e_dark.basic.exposure_ev
                );
            }
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
            || e.basic.dehaze != 0.0
        {
            panic!("auto set non-owned basic fields: {:?}", e.basic);
        }
    }
}
