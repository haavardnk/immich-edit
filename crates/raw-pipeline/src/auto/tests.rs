use super::*;
use crate::edits::{CropRect, GeometryEdits, LensEdits};
use crate::frame::{FrameId, FrameMeta, OutputFormat, RenderOptions};

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
        meta: FrameMeta {
            width: w,
            height: h,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: cfa.to_string(),
        bps: 16,
        data,
        cpp: 1,
        exif: None,
        id: FrameId::fresh(),
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
        meta: FrameMeta {
            width: w,
            height: h,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data,
        cpp: 3,
        exif: None,
        id: FrameId::fresh(),
    }
}

fn make_ramp(mid: f32, w: usize, h: usize) -> RawFrame {
    make_frame_with(w, h, |x, y| {
        let u = (x as f32 + 0.5) / w as f32;
        let v = (y as f32 + 0.5) / h as f32;
        (mid * 2f32.powf((u - 0.5) * 6.0 + (v - 0.5))).clamp(0.0, 1.0)
    })
}

fn auto(frame: &RawFrame, context: &Edits) -> Edits {
    auto_with(frame, context, &[])
}

fn auto_with(frame: &RawFrame, context: &Edits, faces: &[Face]) -> Edits {
    match auto_adjust(frame, context, None, faces) {
        Ok(edits) => edits,
        Err(err) => panic!("auto failed: {err}"),
    }
}

fn solve_at(frame: &RawFrame, exposure_value: Option<f64>) -> solve::Sliders {
    let solved = proxy::Proxy::new(frame, Edits::default(), None, &[])
        .and_then(|mut proxy| solve::solve(&mut proxy, exposure_value));
    match solved {
        Ok(sliders) => sliders,
        Err(err) => panic!("solve failed: {err}"),
    }
}

#[test]
fn exposure_follows_scene_brightness() {
    let exposures: Vec<f64> = [0.01, 0.05, 0.18, 0.5]
        .iter()
        .map(|&mid| {
            auto(&make_ramp(mid, 96, 64), &Edits::default())
                .basic
                .exposure_ev
        })
        .collect();
    if exposures.windows(2).any(|w| w[1] >= w[0]) {
        panic!("exposure should fall as the scene brightens: {exposures:?}");
    }
    if exposures[2].abs() > 0.7 {
        panic!("mid-grey scene should need little exposure: {exposures:?}");
    }
}

#[test]
fn lifting_a_dark_scene_keeps_bright_detail() {
    let f = make_frame_with(64, 64, |x, y| {
        if x < 12 {
            0.6 + 0.3 * y as f32 / 64.0
        } else {
            0.04
        }
    });
    let e = auto(&f, &Edits::default());
    if e.basic.exposure_ev <= 0.0 {
        panic!(
            "expected some lift for dark scene, ev={}",
            e.basic.exposure_ev
        );
    }
    let options = RenderOptions {
        max_edge: 64,
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    };
    let out = match crate::cpu::pipeline::render(&f, &e, &options) {
        Ok(out) => out,
        Err(err) => panic!("render failed: {err}"),
    };
    let width = out.width as usize;
    let clipped = (0..64)
        .filter(|&y| out.bytes[(y * width + 4) * 3] >= 254)
        .count();
    if clipped > 4 {
        panic!(
            "auto blew {clipped} of 64 bright rows at ev={}: {:?}",
            e.basic.exposure_ev, e.tone
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
    let with_lens = auto(&f, &edits);
    let without_lens = auto(&f, &Edits::default());
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
    let without = auto(&f, &Edits::default());
    let with_crop = auto(&f, &cropped);
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
            let want = auto(&rgb, &context);
            let got = auto(&mosaic, &context);
            if (got.basic.exposure_ev - want.basic.exposure_ev).abs() > 0.1 {
                panic!(
                    "cfa '{cfa}': mosaic exposure {} disagrees with demosaiced {}",
                    got.basic.exposure_ev, want.basic.exposure_ev
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
        let e_dark = auto(&dark, &Edits::default());
        let e_bright = auto(&bright, &Edits::default());
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
fn auto_owns_tone_and_preserves_the_rest() {
    for bw in [false, true] {
        let mut context = Edits::default();
        context.basic.exposure_ev = 3.0;
        context.basic.brightness = 30.0;
        context.basic.texture = 20.0;
        context.basic.saturation = -40.0;
        context.detail.luma_nr_amount = 30.0;
        context.color.bw.enabled = bw;
        let e = auto(&make_ramp(0.18, 96, 64), &context);
        if e.basic.exposure_ev >= 2.0 || e.basic.brightness != 0.0 {
            panic!("bw={bw}: auto kept stale exposure fields: {:?}", e.basic);
        }
        if e.basic.texture != 20.0 || e.detail.luma_nr_amount != 30.0 || e.color.bw.enabled != bw {
            panic!("bw={bw}: auto dropped unowned fields: {e:?}");
        }
        if bw && e.basic.saturation != -40.0 {
            panic!("auto changed B&W saturation: {}", e.basic.saturation);
        }
        if !bw && e.basic.saturation < 0.0 {
            panic!("auto kept colour saturation: {}", e.basic.saturation);
        }
    }
}

#[test]
fn faces_pull_exposure_toward_the_face() {
    let f = make_frame_with(128, 96, |x, _| if x < 48 { 0.02 } else { 0.35 });
    let face = Face {
        x: 0.08,
        y: 0.3,
        w: 0.2,
        h: 0.35,
    };
    let without = auto(&f, &Edits::default());
    let with = auto_with(&f, &Edits::default(), &[face]);
    if with.basic.exposure_ev < without.basic.exposure_ev + 0.2 {
        panic!(
            "a dark face should lift exposure; with={} without={}",
            with.basic.exposure_ev, without.basic.exposure_ev
        );
    }
    if with.basic.clarity > 8.0 {
        panic!("portrait clarity too strong: {}", with.basic.clarity);
    }
}

#[test]
fn night_scene_keeps_a_darker_key() {
    let f = make_ramp(0.05, 96, 64);
    let day = solve_at(&f, Some(12.0));
    let night = solve_at(&f, Some(0.0));
    if night.exposure > day.exposure - 0.2 {
        panic!(
            "night exposure {} should stay below day {}",
            night.exposure, day.exposure
        );
    }
}

#[test]
fn haze_drives_dehaze() {
    let hazy = make_frame_with(128, 96, |x, y| 0.3 + 0.25 * (x + y) as f32 / 224.0);
    let clear = make_frame_with(128, 96, |x, y| {
        if x % 3 == 0 || y % 3 == 0 {
            0.002
        } else {
            0.3 + 0.25 * (x + y) as f32 / 224.0
        }
    });
    let hazy = auto(&hazy, &Edits::default()).basic.dehaze;
    let clear = auto(&clear, &Edits::default()).basic.dehaze;
    if hazy < 10.0 || clear != 0.0 {
        panic!("dehaze should follow the veil; hazy={hazy} clear={clear}");
    }
}
