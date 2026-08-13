use super::*;

#[test]
fn texture_positive_amplifies_detail() {
    let w: usize = 256;
    let h: usize = 128;
    let period: usize = 8;
    let mk = || {
        let mut buf = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let stripe = (((x / (period / 2)) % 2) as f32) * 0.1 + 0.45;
                let i = (y * w + x) * 3;
                buf[i] = stripe;
                buf[i + 1] = stripe;
                buf[i + 2] = stripe;
            }
        }
        LinearImage::new(buf, w, h)
    };
    let edits_pos = Edits {
        basic: BasicEdits {
            texture: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let edits_neg = Edits {
        basic: BasicEdits {
            texture: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let probe_hi = (h / 2 * w + period / 2) * 3;
    let probe_lo = (h / 2 * w + period) * 3;
    let base = mk();
    let base_spread = (base.rgb[probe_hi] - base.rgb[probe_lo]).abs();
    let mut pos = mk();
    texture::TextureOp
        .apply_cpu(&mut pos, &ctx(), &edits_pos)
        .unwrap();
    let pos_spread = (pos.rgb[probe_hi] - pos.rgb[probe_lo]).abs();
    let mut neg = mk();
    texture::TextureOp
        .apply_cpu(&mut neg, &ctx(), &edits_neg)
        .unwrap();
    let neg_spread = (neg.rgb[probe_hi] - neg.rgb[probe_lo]).abs();
    assert!(
        pos_spread > base_spread * 1.2,
        "texture +100 should amplify mid-frequency: base={base_spread} pos={pos_spread}"
    );
    assert!(
        neg_spread < base_spread * 0.9,
        "texture -100 should reduce mid-frequency: base={base_spread} neg={neg_spread}"
    );
}

#[test]
fn clarity_amplifies_midtones_more_than_extremes() {
    let mk = |left: f32, right: f32| {
        let mut buf = vec![0.0f32; 200 * 10 * 3];
        for y in 0..10 {
            for x in 0..200 {
                let v = if x < 100 { left } else { right };
                let i = (y * 200 + x) * 3;
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
            }
        }
        LinearImage::new(buf, 200, 10)
    };
    let edits = Edits {
        basic: BasicEdits {
            clarity: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut mid = mk(0.45, 0.55);
    let mut bright = mk(0.92, 1.00);
    let probe = (5 * 200 + 99) * 3;
    let mid_before = mid.rgb[probe];
    let bright_before = bright.rgb[probe];
    clarity::ClarityOp
        .apply_cpu(&mut mid, &ctx(), &edits)
        .unwrap();
    clarity::ClarityOp
        .apply_cpu(&mut bright, &ctx(), &edits)
        .unwrap();
    let mid_delta = (mid.rgb[probe] - mid_before).abs();
    let bright_delta = (bright.rgb[probe] - bright_before).abs();
    assert!(mid_delta > bright_delta);
}

#[test]
fn clarity_flat_region_barely_changes() {
    let mut img = LinearImage::new(vec![0.5f32; 256 * 256 * 3], 256, 256);
    let before = img.rgb.clone();
    let edits = Edits {
        basic: BasicEdits {
            clarity: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    clarity::ClarityOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let max_d = before
        .iter()
        .zip(img.rgb.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    if max_d > 5e-3 {
        panic!("clarity boosted flat patch: max_d={max_d}");
    }
}

#[test]
fn clarity_protects_clipped_highlights_and_crushed_shadows() {
    let mk = |v: f32| LinearImage::new(vec![v; 256 * 256 * 3], 256, 256);
    let edits = Edits {
        basic: BasicEdits {
            clarity: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    for v in [1.2f32, 0.02f32] {
        let mut img = mk(v);
        let before = img.rgb.clone();
        clarity::ClarityOp
            .apply_cpu(&mut img, &ctx(), &edits)
            .unwrap();
        let max_d = before
            .iter()
            .zip(img.rgb.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        if max_d > 1e-3 {
            panic!("clarity moved protected patch v={v}: max_d={max_d}");
        }
    }
}

#[test]
fn dehaze_atmosphere_is_resolution_invariant() {
    let pattern = |w: usize, h: usize| -> Vec<f32> {
        (0..w * h * 3)
            .map(|i| {
                let px = i / 3;
                let x = (px % w) as f32 / w as f32;
                let y = (px / w) as f32 / h as f32;
                let haze = 0.55 + 0.35 * y;
                let detail = ((x * 47.0).sin() * (y * 31.0).cos()).abs() * 0.25;
                (haze + detail).min(1.0)
            })
            .collect()
    };
    let big = crate::cpu::dehaze::atmosphere_for_render(&pattern(1200, 900), 1200, 900);
    let small = crate::cpu::dehaze::atmosphere_for_render(&pattern(600, 450), 600, 450);
    let max_d = big
        .iter()
        .zip(small.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    if max_d > 0.02 {
        panic!("atmosphere depends on render size: {big:?} vs {small:?}");
    }
}

#[test]
fn dehaze_zero_is_identity() {
    let mut img = LinearImage::new(
        (0..64 * 64 * 3).map(|i| (i % 100) as f32 / 100.0).collect(),
        64,
        64,
    );
    let before = img.rgb.clone();
    let edits = Edits::default();
    dehaze::DehazeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let max_d = before
        .iter()
        .zip(img.rgb.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    if max_d > 1e-6 {
        panic!("dehaze zero changed image: {max_d}");
    }
}

#[test]
fn dehaze_positive_recovers_synthetic_haze() {
    let w: usize = 256;
    let h: usize = 64;
    let atm = [0.9f32, 0.9, 0.9];
    let j_dark = [0.2f32, 0.1, 0.05];
    let t = 0.5f32;
    let dark_px = [
        j_dark[0] * t + atm[0] * (1.0 - t),
        j_dark[1] * t + atm[1] * (1.0 - t),
        j_dark[2] * t + atm[2] * (1.0 - t),
    ];
    let sky_px = atm;
    let mut buf = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let p = if x < w / 2 { dark_px } else { sky_px };
            buf[i] = p[0];
            buf[i + 1] = p[1];
            buf[i + 2] = p[2];
        }
    }
    let mut img = LinearImage::new(buf, w, h);
    let edits = Edits {
        basic: BasicEdits {
            dehaze: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    dehaze::DehazeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let i = (h / 2 * w + w / 4) * 3;
    let dr = (img.rgb[i] - j_dark[0]).abs();
    let dg = (img.rgb[i + 1] - j_dark[1]).abs();
    let db = (img.rgb[i + 2] - j_dark[2]).abs();
    if dr > 0.1 || dg > 0.1 || db > 0.1 {
        panic!(
            "dehaze did not recover J: r={} g={} b={} (truth {:?})",
            img.rgb[i],
            img.rgb[i + 1],
            img.rgb[i + 2],
            j_dark
        );
    }
}

#[test]
fn dehaze_negative_pushes_toward_atmosphere() {
    let w: usize = 64;
    let h: usize = 64;
    let mut buf = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            buf[i] = 0.4;
            buf[i + 1] = 0.5;
            buf[i + 2] = 0.6;
        }
    }
    let mut img = LinearImage::new(buf, w, h);
    let edits = Edits {
        basic: BasicEdits {
            dehaze: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    dehaze::DehazeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let i = (h / 2 * w + w / 2) * 3;
    if img.rgb[i] <= 0.4 {
        panic!(
            "negative dehaze did not lift toward atmosphere: r={}",
            img.rgb[i]
        );
    }
}

#[test]
fn sharpen_amplifies_edge_contrast() {
    let w: usize = 64;
    let h: usize = 16;
    let mk = || {
        let mut buf = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let v = if x < w / 2 { 0.3 } else { 0.7 };
                let i = (y * w + x) * 3;
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
            }
        }
        LinearImage::new(buf, w, h)
    };
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(100.0),
            sharpen_radius: 1.0,
            sharpen_detail: 100.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let base = mk();
    let base_left = base.rgb[(8 * w + (w / 2 - 1)) * 3];
    let base_right = base.rgb[(8 * w + (w / 2)) * 3];
    let mut img = mk();
    sharpen::SharpenOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let left = img.rgb[(8 * w + (w / 2 - 1)) * 3];
    let right = img.rgb[(8 * w + (w / 2)) * 3];
    assert!(
        left < base_left && right > base_right,
        "expected sharper edge: base=({base_left},{base_right}) got=({left},{right})"
    );
}

#[test]
fn sharpen_masking_suppresses_flat_areas() {
    let w: usize = 64;
    let h: usize = 32;
    let mut buf = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let n = ((x * 7 + y * 13) % 11) as f32 / 1000.0;
            let v = 0.5 + n;
            let i = (y * w + x) * 3;
            buf[i] = v;
            buf[i + 1] = v;
            buf[i + 2] = v;
        }
    }
    let img_no_mask = LinearImage::new(buf.clone(), w, h);
    let img_mask = LinearImage::new(buf.clone(), w, h);
    let mk_edits = |masking: f64| Edits {
        detail: DetailEdits {
            sharpen_amount: Some(150.0),
            sharpen_radius: 1.0,
            sharpen_detail: 100.0,
            sharpen_masking: masking,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut a = img_no_mask;
    let mut b = img_mask;
    sharpen::SharpenOp
        .apply_cpu(&mut a, &ctx(), &mk_edits(0.0))
        .unwrap();
    sharpen::SharpenOp
        .apply_cpu(&mut b, &ctx(), &mk_edits(100.0))
        .unwrap();
    let diff_no_mask: f32 = a
        .rgb
        .iter()
        .zip(buf.iter())
        .map(|(x, y)| (x - y).abs())
        .sum();
    let diff_mask: f32 = b
        .rgb
        .iter()
        .zip(buf.iter())
        .map(|(x, y)| (x - y).abs())
        .sum();
    assert!(
        diff_mask < diff_no_mask * 0.25,
        "masking should suppress noise sharpening: no_mask={diff_no_mask} masked={diff_mask}"
    );
}
