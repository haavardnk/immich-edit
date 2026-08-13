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
