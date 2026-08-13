use super::*;
use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_segment};
use crate::edits::{LensEdits, MaskComponent, MaskSource, Vec2f};
use crate::ops::LinearImage;
use crate::ops::lens_distortion::{LensWarpParams, mask_uv_to_scene_uv};

fn linear(id: &str, p0: Vec2f, p1: Vec2f, feather: f32) -> MaskComponent {
    MaskComponent {
        id: id.into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Linear { p0, p1, feather },
        source: MaskSource::Manual,
        generated: None,
    }
}

#[test]
fn linear_gradient_weights() {
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![linear(
            "c",
            Vec2f { x: 0.0, y: 0.0 },
            Vec2f { x: 1.0, y: 0.0 },
            1.0,
        )],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let w_left = fold_layer_weight(&eval, 0.0, 0.5);
    let w_right = fold_layer_weight(&eval, 1.0, 0.5);
    let w_mid = fold_layer_weight(&eval, 0.5, 0.5);
    if w_left > 0.05 {
        panic!("expected 0 at p0, got {w_left}");
    }
    if w_right < 0.95 {
        panic!("expected 1 at p1, got {w_right}");
    }
    if (w_mid - 0.5).abs() > 0.1 {
        panic!("expected ~0.5 at mid, got {w_mid}");
    }
}

#[test]
fn radial_inside_outside() {
    let mut comp = linear("c", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 0.0, y: 0.0 }, 0.0);
    comp.kind = MaskComponentKind::Radial {
        center: Vec2f { x: 0.5, y: 0.5 },
        radius_xy: Vec2f { x: 0.2, y: 0.2 },
        feather: 0.1,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![comp],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let inside = fold_layer_weight(&eval, 0.5, 0.5);
    let outside = fold_layer_weight(&eval, 0.9, 0.9);
    if inside < 0.95 {
        panic!("expected ~1 inside, got {inside}");
    }
    if outside > 0.05 {
        panic!("expected ~0 outside, got {outside}");
    }
}

#[test]
fn layer_invert_flips_the_folded_weight() {
    let mut comp = linear("c", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 0.0, y: 0.0 }, 0.0);
    comp.kind = MaskComponentKind::Radial {
        center: Vec2f { x: 0.5, y: 0.5 },
        radius_xy: Vec2f { x: 0.2, y: 0.2 },
        feather: 0.1,
    };
    let mut layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: true,
        components: vec![comp],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let inside = fold_layer_weight(&eval, 0.5, 0.5);
    let outside = fold_layer_weight(&eval, 0.9, 0.9);
    if inside > 0.05 {
        panic!("expected inverted centre cleared, got {inside}");
    }
    if outside < 0.95 {
        panic!("expected inverted surround selected, got {outside}");
    }
    layer.amount = 0.5;
    let scaled = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let half = fold_layer_weight(&scaled, 0.9, 0.9);
    if (half - 0.5).abs() > 0.05 {
        panic!("expected amount to scale after invert, got {half}");
    }
}

#[test]
fn subtract_carves_out() {
    let add = linear("a", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 1.0, y: 0.0 }, 0.0);
    let mut sub = linear("s", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 1.0, y: 0.0 }, 0.0);
    sub.mode = MaskComponentMode::Subtract;
    sub.kind = MaskComponentKind::Radial {
        center: Vec2f { x: 0.5, y: 0.5 },
        radius_xy: Vec2f { x: 0.1, y: 0.1 },
        feather: 0.05,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![add, sub],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let on_carve = fold_layer_weight(&eval, 0.5, 0.5);
    let right_clear = fold_layer_weight(&eval, 0.95, 0.5);
    if on_carve > 0.1 {
        panic!("expected near-zero where subtracted, got {on_carve}");
    }
    if right_clear < 0.85 {
        panic!("expected ~1 outside carve, got {right_clear}");
    }
}

#[test]
fn masked_exposure_brightens_only_right_half() {
    let w = 8;
    let h = 4;
    let rgb = vec![0.5f32; w * h * 3];
    let mut image = LinearImage::new(rgb, w, h);
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![linear(
            "c",
            Vec2f { x: 0.0, y: 0.0 },
            Vec2f { x: 1.0, y: 0.0 },
            0.0,
        )],
        edits: crate::edits::MaskedEdits {
            exposure_ev: Some(2.0),
            ..Default::default()
        },
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let mut layer_seg = FusedSegment::default();
    layer_seg.push(CpuFusedOp::Exposure { factor: 4.0 });
    let mut layer_image = LinearImage::new(image.rgb.clone(), w, h);
    apply_segment(&mut layer_image, &layer_seg);
    let warp = LensWarpParams::from_edits(&Default::default(), w as u32, h as u32);
    blend_layer_images(&mut image, &[layer_image], &[eval], &warp);
    let left = image.rgb[0];
    let right = image.rgb[3 * (w - 1)];
    if (left - 0.5).abs() > 1e-3 {
        panic!("expected left untouched, got {left}");
    }
    if (right - 2.0).abs() > 1e-3 {
        panic!("expected right ~2.0 (4x base), got {right}");
    }
}

#[test]
fn sharpen_delta_image_follows_mask_weight() {
    let w = 8;
    let h = 4;
    let image = LinearImage::new(vec![0.5f32; w * h * 3], w, h);
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![linear(
            "c",
            Vec2f { x: 0.0, y: 0.0 },
            Vec2f { x: 1.0, y: 0.0 },
            0.0,
        )],
        edits: crate::edits::MaskedEdits {
            sharpen: Some(100.0),
            ..Default::default()
        },
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let warp = LensWarpParams::from_edits(&Default::default(), w as u32, h as u32);
    let delta = build_sharpen_delta_image(&image, &[eval], &[100.0], &warp);
    let left = delta.rgb[0];
    let right = delta.rgb[3 * (w - 1)];
    if left.abs() > 1e-3 {
        panic!("expected zero delta on the left, got {left}");
    }
    if (right - 100.0).abs() > 1e-3 {
        panic!("expected full delta on the right, got {right}");
    }
}

#[test]
fn render_mask_overlay_preserves_context_and_marks_selection_red() {
    let w = 16;
    let h = 4;
    let mut image = LinearImage::new(vec![4.0f32; w * h * 3], w, h);
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![linear(
            "c",
            Vec2f { x: 0.0, y: 0.0 },
            Vec2f { x: 1.0, y: 0.0 },
            1.0,
        )],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let warp = LensWarpParams::from_edits(&Default::default(), w as u32, h as u32);
    render_mask_overlay(&mut image, &eval, &warp, None);
    let left = [image.rgb[0], image.rgb[1], image.rgb[2]];
    let right_index = 3 * (w - 1);
    let right = [
        image.rgb[right_index],
        image.rgb[right_index + 1],
        image.rgb[right_index + 2],
    ];
    if left.iter().any(|value| !(0.0..=1.0).contains(value)) {
        panic!("expected bounded image context, got {left:?}");
    }
    if right[0] <= right[1] || right[0] <= right[2] {
        panic!("expected selected area shifted toward red, got {right:?}");
    }
    if right[1] >= left[1] || right[2] >= left[2] {
        panic!("expected overlay to reduce green and blue, got {left:?} and {right:?}");
    }
}

#[test]
fn brush_raster_samples_bilinear() {
    let bytes = vec![0u8, 0, 255, 255];
    let raster = Arc::new(MaskRaster::new(2, 2, bytes).unwrap());
    let mut rasters = RasterMap::new();
    rasters.insert("r1".into(), raster);
    let comp = MaskComponent {
        id: "c".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Brush {
            raster_id: "r1".into(),
        },
        source: MaskSource::Manual,
        generated: None,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![comp],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &rasters);
    let tl = fold_layer_weight(&eval, 0.0, 0.0);
    let br = fold_layer_weight(&eval, 1.0, 1.0);
    let mid = fold_layer_weight(&eval, 0.5, 0.5);
    if tl > 0.05 {
        panic!("expected ~0 at (0,0), got {tl}");
    }
    if br < 0.95 {
        panic!("expected ~1 at (1,1), got {br}");
    }
    if (mid - 0.5).abs() > 0.1 {
        panic!("expected ~0.5 at center, got {mid}");
    }
}

fn polygon_layer(points: Vec<Vec2f>, feather: f32) -> MaskLayer {
    MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Polygon { points, feather },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: Default::default(),
    }
}

#[test]
fn polygon_selects_inside_and_rejects_outside() {
    let square = vec![
        Vec2f { x: 0.2, y: 0.2 },
        Vec2f { x: 0.8, y: 0.2 },
        Vec2f { x: 0.8, y: 0.8 },
        Vec2f { x: 0.2, y: 0.8 },
    ];
    let eval = build_layer_eval(&polygon_layer(square, 0.0), &RasterMap::new());
    let inside = fold_layer_weight(&eval, 0.5, 0.5);
    let outside = fold_layer_weight(&eval, 0.05, 0.5);
    let corner = fold_layer_weight(&eval, 0.9, 0.9);
    if inside < 0.99 {
        panic!("expected inside selected, got {inside}");
    }
    if outside > 1e-6 {
        panic!("expected outside rejected, got {outside}");
    }
    if corner > 1e-6 {
        panic!("expected corner rejected, got {corner}");
    }
}

#[test]
fn polygon_handles_a_concave_shape() {
    let arrow = vec![
        Vec2f { x: 0.1, y: 0.1 },
        Vec2f { x: 0.9, y: 0.1 },
        Vec2f { x: 0.5, y: 0.5 },
        Vec2f { x: 0.9, y: 0.9 },
        Vec2f { x: 0.1, y: 0.9 },
    ];
    let eval = build_layer_eval(&polygon_layer(arrow, 0.0), &RasterMap::new());
    let filled = fold_layer_weight(&eval, 0.2, 0.5);
    let notch = fold_layer_weight(&eval, 0.8, 0.5);
    if filled < 0.99 {
        panic!("expected filled side selected, got {filled}");
    }
    if notch > 1e-6 {
        panic!("expected concave notch rejected, got {notch}");
    }
}

#[test]
fn polygon_feather_softens_the_inner_edge() {
    let square = vec![
        Vec2f { x: 0.2, y: 0.2 },
        Vec2f { x: 0.8, y: 0.2 },
        Vec2f { x: 0.8, y: 0.8 },
        Vec2f { x: 0.2, y: 0.8 },
    ];
    let eval = build_layer_eval(&polygon_layer(square, 0.2), &RasterMap::new());
    let centre = fold_layer_weight(&eval, 0.5, 0.5);
    let near_edge = fold_layer_weight(&eval, 0.25, 0.5);
    if centre < 0.99 {
        panic!("expected centre fully selected, got {centre}");
    }
    if !(1e-6..0.99).contains(&near_edge) {
        panic!("expected softened edge, got {near_edge}");
    }
}

#[test]
fn polygon_under_three_points_is_empty() {
    let line = vec![Vec2f { x: 0.2, y: 0.2 }, Vec2f { x: 0.8, y: 0.8 }];
    let eval = build_layer_eval(&polygon_layer(line, 0.0), &RasterMap::new());
    let w = fold_layer_weight(&eval, 0.5, 0.5);
    if w > 1e-6 {
        panic!("expected zero weight for a degenerate polygon, got {w}");
    }
}

#[test]
fn brush_missing_raster_yields_zero_weight() {
    let comp = MaskComponent {
        id: "c".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Brush {
            raster_id: "missing".into(),
        },
        source: MaskSource::Manual,
        generated: None,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![comp],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &RasterMap::new());
    let w = fold_layer_weight(&eval, 0.5, 0.5);
    if w > 1e-6 {
        panic!("expected 0 with missing raster, got {w}");
    }
}

#[test]
fn luma_range_selects_and_softens_boundaries() {
    let component = MaskComponent {
        id: "c".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::LumaRange {
            min: 0.4,
            max: 0.6,
            softness: 0.2,
        },
        source: MaskSource::Manual,
        generated: None,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![component],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &RasterMap::new());
    let inside = fold_layer_weight_with_display(&eval, 0.5, 0.5, [0.5, 0.5, 0.5]);
    let soft = fold_layer_weight_with_display(&eval, 0.5, 0.5, [0.3, 0.3, 0.3]);
    let outside = fold_layer_weight_with_display(&eval, 0.5, 0.5, [0.1, 0.1, 0.1]);
    if inside < 0.99 {
        panic!("expected inside luma selected, got {inside}");
    }
    if !(0.0..1.0).contains(&soft) {
        panic!("expected soft luma boundary, got {soft}");
    }
    if outside > 1e-6 {
        panic!("expected outside luma rejected, got {outside}");
    }
}

#[test]
fn color_range_prefers_sampled_color() {
    let component = MaskComponent {
        id: "c".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::ColorRange {
            sample_rgb: [0.9, 0.1, 0.1],
            tolerance: 0.05,
            softness: 0.05,
        },
        source: MaskSource::Manual,
        generated: None,
    };
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![component],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &RasterMap::new());
    let red = fold_layer_weight_with_display(&eval, 0.5, 0.5, [0.9, 0.1, 0.1]);
    let blue = fold_layer_weight_with_display(&eval, 0.5, 0.5, [0.1, 0.1, 0.9]);
    if red < 0.99 {
        panic!("expected sampled red selected, got {red}");
    }
    if blue > 0.01 {
        panic!("expected blue rejected, got {blue}");
    }
}

#[test]
fn effective_edits_adds_and_clamps_brightness() {
    let mut g = Edits::default();
    g.basic.brightness = 80.0;
    let mut layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![],
        edits: Default::default(),
    };
    layer.edits.brightness = Some(50.0);
    let eff = effective_edits_for_layer(&g, &layer);
    if (eff.basic.brightness - 100.0).abs() > 1e-6 {
        panic!("expected clamp to 100, got {}", eff.basic.brightness);
    }
    layer.edits.brightness = Some(-200.0);
    let eff = effective_edits_for_layer(&g, &layer);
    if (eff.basic.brightness - (-100.0)).abs() > 1e-6 {
        panic!("expected clamp to -100, got {}", eff.basic.brightness);
    }
}

#[test]
fn scene_space_mask_anchors_through_lens_warp() {
    let w = 64usize;
    let h = 32usize;
    let layer = MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![linear(
            "c",
            Vec2f { x: 0.0, y: 0.0 },
            Vec2f { x: 1.0, y: 0.0 },
            0.0,
        )],
        edits: Default::default(),
    };
    let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
    let identity = LensWarpParams::from_edits(&LensEdits::default(), w as u32, h as u32);
    let lens = LensEdits {
        profile_enabled: Some(true),
        k1: -0.15,
        constrain_crop: true,
        ..Default::default()
    };
    let warp = LensWarpParams::from_edits(&lens, w as u32, h as u32);
    if warp.is_identity() {
        panic!("warp should be non-identity for anchoring test");
    }
    let mut img_warp = LinearImage::new(vec![0.0f32; w * h * 3], w, h);
    render_mask_overlay(&mut img_warp, &eval, &warp, None);
    let mut img_id = LinearImage::new(vec![0.0f32; w * h * 3], w, h);
    render_mask_overlay(&mut img_id, &eval, &identity, None);
    let samples = [(0.5, 0.5), (0.7, 0.5), (0.5, 0.3), (0.8, 0.7)];
    for (u, v) in samples {
        let mx = ((u * w as f32) as usize).min(w - 1);
        let my = ((v * h as f32) as usize).min(h - 1);
        let warped = img_warp.rgb[(my * w + mx) * 3];
        let scene = mask_uv_to_scene_uv(&warp, [u, v]);
        let sx = ((scene[0] * w as f32) as usize).min(w - 1);
        let sy = ((scene[1] * h as f32) as usize).min(h - 1);
        let identity_at_scene = img_id.rgb[(sy * w + sx) * 3];
        if (warped - identity_at_scene).abs() > 0.05 {
            panic!(
                "anchor mismatch at ({u},{v}): warped={warped} identity_at_scene={identity_at_scene}"
            );
        }
    }
}
