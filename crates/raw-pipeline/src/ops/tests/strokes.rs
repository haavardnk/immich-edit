use super::*;

fn split_image(w: usize, h: usize) -> LinearImage {
    let mut buf = vec![0.0f32; w * h * 3];
    for (i, px) in buf.chunks_mut(3).enumerate() {
        px.fill(if (i % w) >= w / 2 { 0.8 } else { 0.2 });
    }
    LinearImage::new(buf, w, h)
}

fn retouched(mode: RetouchMode) -> LinearImage {
    let mut img = split_image(64, 64);
    let edits = Edits {
        retouch: vec![RetouchStroke {
            id: "s".into(),
            mode,
            points: vec![Vec2f { x: 0.25, y: 0.5 }],
            radius: 0.05,
            hardness: 1.0,
            opacity: 1.0,
            source: Vec2f { x: 0.75, y: 0.5 },
            enabled: true,
        }],
        ..Default::default()
    };
    retouch::RetouchOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    img
}

#[test]
fn retouch_clone_takes_source_heal_keeps_destination_tone() {
    for (mode, expected) in [(RetouchMode::Clone, 0.8f32), (RetouchMode::Heal, 0.2f32)] {
        let img = retouched(mode);
        let center = (32 * 64 + 16) * 3;
        let outside = (32 * 64 + 2) * 3;
        let got = img.rgb[center];
        if (got - expected).abs() > 1e-3 {
            panic!("{mode:?} center {got} expected {expected}");
        }
        if (img.rgb[outside] - 0.2).abs() > 1e-6 {
            panic!("{mode:?} leaked outside stroke radius");
        }
    }
}

#[test]
fn retouch_source_patch_stays_inside_the_frame() {
    for (name, source) in [
        ("above", Vec2f { x: 0.5, y: -0.4 }),
        ("below", Vec2f { x: 0.5, y: 1.4 }),
        ("left", Vec2f { x: -0.4, y: 0.5 }),
        ("right", Vec2f { x: 1.4, y: 0.5 }),
    ] {
        let stroke = RetouchStroke {
            id: name.into(),
            mode: RetouchMode::Clone,
            points: vec![Vec2f { x: 0.5, y: 0.5 }],
            radius: 0.1,
            hardness: 1.0,
            opacity: 1.0,
            source,
            enabled: true,
        };
        let geom = retouch::stroke_geometry(&stroke, 200, 200, (false, false, false)).unwrap();
        let x0 = geom.bbox.x0 as f32 + geom.off_x;
        let x1 = geom.bbox.x1 as f32 + geom.off_x;
        let y0 = geom.bbox.y0 as f32 + geom.off_y;
        let y1 = geom.bbox.y1 as f32 + geom.off_y;
        if x0 < 0.0 || y0 < 0.0 || x1 > 200.0 || y1 > 200.0 {
            panic!("{name}: sampled patch {x0}..{x1} x {y0}..{y1} leaves the frame");
        }
    }
}

fn blemish_scene(size: usize, blemish_px: f32) -> LinearImage {
    let s = size as f32;
    let buf = (0..size * size)
        .flat_map(|i| {
            let x = (i % size) as f32 + 0.5;
            let y = (i / size) as f32 + 0.5;
            let d2 = (x - 0.3 * s).powi(2) + (y - 0.4 * s).powi(2);
            let blemish = if blemish_px > 0.0 {
                0.2 * (-d2 / (2.0 * blemish_px * blemish_px)).exp()
            } else {
                0.0
            };
            let v = 0.3 + 0.2 * x / s + 0.1 * (y / s).powi(2) - blemish;
            [v, v * 0.8, v * 0.6]
        })
        .collect();
    LinearImage::new(buf, size, size)
}

#[test]
fn heal_removes_the_blemish_under_the_stroke() {
    let size = 200;
    let clean = blemish_scene(size, 0.0);
    let error = |img: &LinearImage| -> f32 {
        img.rgb
            .iter()
            .zip(&clean.rgb)
            .map(|(a, b)| (a - b).abs())
            .sum()
    };
    for radius in [0.05f32, 0.1, 0.2] {
        let blemish_px = 0.35 * radius * size as f32;
        let blemished = blemish_scene(size, blemish_px);
        let mut healed = blemish_scene(size, blemish_px);
        let edits = Edits {
            retouch: vec![RetouchStroke {
                id: "h".into(),
                mode: RetouchMode::Heal,
                points: vec![Vec2f { x: 0.3, y: 0.4 }],
                radius,
                hardness: 1.0,
                opacity: 1.0,
                source: Vec2f { x: 0.7, y: 0.6 },
                enabled: true,
            }],
            ..Default::default()
        };
        retouch::RetouchOp
            .apply_cpu(&mut healed, &ctx(), &edits)
            .unwrap();
        let residual = error(&healed) / error(&blemished);
        if residual > 0.1 {
            panic!(
                "radius {radius}: {:.0}% of the blemish remains",
                residual * 100.0
            );
        }
    }
}
