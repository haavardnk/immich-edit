use super::*;

#[test]
fn default_is_identity() {
    let e = Edits::default();
    assert!(e.is_identity());
}

#[test]
fn clamp_exposure() {
    let mut e = Edits::default();
    e.basic.exposure_ev = 10.0;
    let c = e.clamped();
    assert_eq!(c.basic.exposure_ev, 5.0);
}

#[test]
fn clamp_invalid_rotate() {
    let mut e = Edits::default();
    e.geometry.rotate = 45;
    let c = e.clamped();
    assert_eq!(c.geometry.rotate, 0);
}

#[test]
fn stable_hash_deterministic() {
    let mut e = Edits::default();
    e.basic.exposure_ev = 1.5;
    let h1 = e.stable_hash();
    let h2 = e.stable_hash();
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 32);
}

#[test]
fn stable_hash_differs_on_change() {
    let mut a = Edits::default();
    a.basic.exposure_ev = 1.0;
    let mut b = Edits::default();
    b.basic.exposure_ev = 2.0;
    assert_ne!(a.stable_hash(), b.stable_hash());
}

#[test]
fn stable_hash_is_pinned() {
    assert_eq!(
        Edits::default().stable_hash(),
        "d1d62a654e4d8d0c21397d451540568a"
    );
    assert_eq!(
        populated_edits().stable_hash(),
        "29f6beebc43c653aac8ce3be54879251"
    );
}

#[test]
fn serde_roundtrip() {
    let mut e = Edits::default();
    e.basic.exposure_ev = 1.0;
    e.geometry.rotate = 90;
    let json = serde_json::to_string(&e).unwrap();
    let e2: Edits = serde_json::from_str(&json).unwrap();
    assert_eq!(e, e2);
}

#[test]
fn serde_roundtrip_populated() {
    let e = populated_edits();
    let json = serde_json::to_string(&e).unwrap();
    let e2: Edits = serde_json::from_str(&json).unwrap();
    assert_eq!(e, e2);
}

#[test]
fn serde_defaults() {
    let json = "{}";
    let e: Edits = serde_json::from_str(json).unwrap();
    assert!(e.is_identity());
}

#[test]
fn mask_brush_serde_roundtrip_preserves_raster_id() {
    let mut e = Edits::default();
    e.masks.push(MaskLayer {
        id: "l1".into(),
        name: "brush layer".into(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "abc123".into(),
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(1.0),
            ..Default::default()
        },
    });
    let json = serde_json::to_string(&e).unwrap();
    let e2: Edits = serde_json::from_str(&json).unwrap();
    assert_eq!(e, e2);
    match &e2.masks[0].components[0].kind {
        MaskComponentKind::Brush { raster_id } => assert_eq!(raster_id, "abc123"),
        _ => panic!("expected brush kind"),
    }
}

#[test]
fn referenced_raster_ids_dedups() {
    let mut e = Edits::default();
    let make_comp = |id: &str, raster: &str| MaskComponent {
        id: id.into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Brush {
            raster_id: raster.into(),
        },
        source: MaskSource::Manual,
        generated: None,
    };
    e.masks.push(MaskLayer {
        id: "l1".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![make_comp("a", "r1"), make_comp("b", "r2")],
        edits: MaskedEdits::default(),
    });
    e.masks.push(MaskLayer {
        id: "l2".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![make_comp("c", "r1")],
        edits: MaskedEdits::default(),
    });
    let ids = e.referenced_raster_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"r1".to_string()));
    assert!(ids.contains(&"r2".to_string()));
}

#[test]
fn retained_raster_ids_includes_probability_maps() {
    let mut e = Edits::default();
    e.masks.push(MaskLayer {
        id: "l1".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "a".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "baked".into(),
            },
            source: MaskSource::Generated,
            generated: Some(GeneratedMeta {
                model_id: "ormbg".into(),
                kind: "subject".into(),
                prob_raster_id: "prob".into(),
                class: None,
                grow: 0.0,
                feather: 0.0,
                painted: false,
                points: Vec::new(),
                range: None,
            }),
        }],
        edits: MaskedEdits::default(),
    });
    assert_eq!(e.referenced_raster_ids(), vec!["baked".to_string()]);
    let retained = e.retained_raster_ids();
    assert_eq!(retained.len(), 2);
    assert!(retained.contains(&"baked".to_string()));
    assert!(retained.contains(&"prob".to_string()));
}

#[test]
fn clamped_preserves_brush_raster_id() {
    let mut e = Edits::default();
    e.masks.push(MaskLayer {
        id: "l".into(),
        name: String::new(),
        enabled: true,
        color: "#fff".into(),
        amount: 2.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "keep-me".into(),
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits::default(),
    });
    let c = e.clamped();
    assert_eq!(c.masks[0].amount, 1.0);
    match &c.masks[0].components[0].kind {
        MaskComponentKind::Brush { raster_id } => assert_eq!(raster_id, "keep-me"),
        _ => panic!("expected brush kind"),
    }
}

fn populated_edits() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 1.25,
            brightness: 10.0,
            contrast: -20.0,
            saturation: 30.0,
            vibrance: -40.0,
            wb_temp: 50.0,
            wb_tint: -60.0,
            texture: 70.0,
            clarity: -80.0,
            dehaze: 90.0,
            curves: CurvesEdits {
                composite: CurvePoints {
                    points: vec![
                        CurvePoint { x: 0.0, y: 0.1 },
                        CurvePoint { x: 0.5, y: 0.4 },
                        CurvePoint { x: 1.0, y: 0.9 },
                    ],
                },
                r: CurvePoints::default(),
                g: CurvePoints::default(),
                b: CurvePoints::default(),
                luma: CurvePoints::default(),
            },
        },
        tone: ToneEdits {
            highlights: -15.0,
            shadows: 25.0,
            blacks: -35.0,
            whites: 45.0,
        },
        color: ColorEdits {
            hsl: HslEdits {
                bands: [HslBand {
                    hue: 5.0,
                    sat: -10.0,
                    lum: 15.0,
                }; HSL_BANDS],
            },
            color_grade: ColorGradeEdits {
                shadows: ColorGradeRegion {
                    hue: 200.0,
                    sat: 20.0,
                    lum: 10.0,
                },
                midtones: ColorGradeRegion {
                    hue: 100.0,
                    sat: 30.0,
                    lum: -10.0,
                },
                highlights: ColorGradeRegion {
                    hue: 40.0,
                    sat: 40.0,
                    lum: 5.0,
                },
                global: ColorGradeRegion {
                    hue: 300.0,
                    sat: 10.0,
                    lum: -5.0,
                },
                balance: 25.0,
                blend: 75.0,
            },
            lut_3d: Lut3dEdits {
                lut_id: Some("lut-1".into()),
                amount: 60.0,
            },
            dcp: DcpEdits {
                mode: DcpMode::Profile,
                profile_id: Some("dcp-1".into()),
                illuminant: crate::dcp::DcpIlluminant::default(),
                use_tone_curve: false,
                use_base_table: true,
                use_look_table: false,
                use_baseline_exposure: true,
            },
        },
        detail: DetailEdits {
            sharpen_amount: 80.0,
            sharpen_radius: 1.5,
            sharpen_detail: 40.0,
            sharpen_masking: 20.0,
            luma_nr_amount: 30.0,
            luma_nr_detail: 60.0,
            luma_nr_contrast: 10.0,
            color_nr_amount: 25.0,
            color_nr_detail: 55.0,
            color_nr_smoothness: 45.0,
        },
        effects: EffectsEdits {
            vignette_amount: -30.0,
            vignette_midpoint: 60.0,
            vignette_feather: 40.0,
            vignette_roundness: 20.0,
            grain_amount: 15.0,
            grain_size: 35.0,
            grain_roughness: 55.0,
        },
        lens: LensEdits {
            profile_enabled: true,
            ca_enabled: true,
            constrain_crop: true,
            distortion_amount: 80.0,
            vignette_amount: 120.0,
            k1: 0.01,
            k2: -0.002,
            k3: 0.0003,
            vk1: 0.1,
            vk2: -0.02,
            vk3: 0.003,
            ca_red_scale_x10000: 12.0,
            ca_blue_scale_x10000: -8.0,
        },
        geometry: GeometryEdits {
            rotate: 90,
            rotate_angle: 3.5,
            flip_h: true,
            flip_v: true,
            crop: Some(CropRect {
                x: 0.1,
                y: 0.2,
                w: 0.5,
                h: 0.6,
            }),
            aspect: AspectLock::Ratio { num: 16, den: 9 },
        },
        masks: vec![MaskLayer {
            id: "layer-1".into(),
            name: "sky".into(),
            enabled: true,
            color: "#00ff00".into(),
            amount: 0.75,
            invert: true,
            components: vec![
                MaskComponent {
                    id: "comp-1".into(),
                    enabled: true,
                    mode: MaskComponentMode::Subtract,
                    invert: true,
                    kind: MaskComponentKind::Linear {
                        p0: Vec2f { x: 0.1, y: 0.2 },
                        p1: Vec2f { x: 0.8, y: 0.9 },
                        feather: 0.3,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
                MaskComponent {
                    id: "comp-2".into(),
                    enabled: true,
                    mode: MaskComponentMode::Intersect,
                    invert: false,
                    kind: MaskComponentKind::Polygon {
                        points: vec![
                            Vec2f { x: 0.0, y: 0.0 },
                            Vec2f { x: 1.0, y: 0.0 },
                            Vec2f { x: 0.5, y: 1.0 },
                        ],
                        feather: 0.2,
                    },
                    source: MaskSource::Generated,
                    generated: Some(GeneratedMeta {
                        model_id: "ormbg".into(),
                        kind: "subject".into(),
                        prob_raster_id: "prob-1".into(),
                        class: Some("sky".into()),
                        grow: 0.4,
                        feather: 0.6,
                        painted: true,
                        points: vec![ClickPointMeta {
                            x: 0.3,
                            y: 0.4,
                            positive: false,
                        }],
                        range: Some(RangeMeta {
                            min: 0.2,
                            max: 0.8,
                            softness: 0.5,
                        }),
                    }),
                },
            ],
            edits: MaskedEdits {
                exposure_ev: Some(0.5),
                brightness: Some(10.0),
                contrast: Some(-10.0),
                saturation: Some(20.0),
                vibrance: Some(-20.0),
                wb_temp: Some(30.0),
                wb_tint: Some(-30.0),
                highlights: Some(40.0),
                shadows: Some(-40.0),
                whites: Some(50.0),
                blacks: Some(-50.0),
                texture: Some(60.0),
                clarity: Some(-60.0),
                sharpen: Some(70.0),
            },
        }],
        retouch: vec![RetouchStroke {
            id: "stroke-1".into(),
            mode: RetouchMode::Clone,
            points: vec![Vec2f { x: 0.2, y: 0.3 }, Vec2f { x: 0.25, y: 0.35 }],
            radius: 0.05,
            hardness: 0.4,
            opacity: 0.9,
            source: Vec2f { x: 0.6, y: 0.7 },
            enabled: true,
        }],
        unknown_ops: std::collections::BTreeMap::from([(
            "future_op".to_string(),
            serde_json::json!({ "amount": 42 }),
        )]),
    }
}
