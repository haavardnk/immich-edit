use super::*;
use crate::edits::{CropRect, GeometryEdits, LensEdits};

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
            profile_enabled: Some(true),
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
