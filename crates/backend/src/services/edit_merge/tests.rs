use super::legacy::{LegacySections, merge_legacy};
use super::*;
use raw_pipeline::edits::{
    BokehShape, DcpMode, LensEdits, Lut3dEdits, RetouchMode, RetouchStroke, ToneEdits, Vec2f,
    WhiteBalance,
};
use serde::Serialize;

fn stroke(id: &str) -> RetouchStroke {
    RetouchStroke {
        id: id.to_string(),
        mode: RetouchMode::Heal,
        points: vec![Vec2f { x: 0.5, y: 0.5 }],
        radius: 0.02,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f { x: 0.6, y: 0.6 },
        enabled: true,
    }
}

fn everything() -> Edits {
    let defaults = Edits::default();
    let mut curves = defaults.basic.curves;
    curves.composite.points[1].y = 0.8;
    let mut hsl = defaults.color.hsl;
    hsl.bands[0].hue = 12.0;
    let mut color_grade = defaults.color.color_grade;
    color_grade.balance = 25.0;
    let mut dcp = defaults.color.dcp;
    dcp.mode = DcpMode::Flat;
    let mut bw = defaults.color.bw;
    bw.enabled = true;
    let mut geometry = defaults.geometry;
    geometry.rotate = 90;
    let mut unknown_ops = defaults.unknown_ops;
    unknown_ops.insert("future_op".to_string(), serde_json::json!({ "x": 2 }));
    Edits {
        basic: BasicEdits {
            exposure_ev: 1.0,
            brightness: 5.0,
            contrast: 6.0,
            saturation: 7.0,
            vibrance: 8.0,
            white_balance: WhiteBalance::Absolute {
                kelvin: 4200.0,
                tint: -20.0,
            },
            texture: 9.0,
            clarity: 15.0,
            dehaze: 10.0,
            curves,
        },
        tone: ToneEdits {
            highlights: -30.0,
            shadows: 20.0,
            blacks: -5.0,
            whites: 4.0,
        },
        color: ColorEdits {
            hsl,
            color_grade,
            lut_3d: Lut3dEdits {
                lut_id: Some("lut".to_string()),
                amount: 70.0,
            },
            dcp,
            bw,
        },
        detail: DetailEdits {
            sharpen_amount: Some(60.0),
            sharpen_radius: 1.7,
            sharpen_detail: 33.0,
            sharpen_masking: 10.0,
            luma_nr_amount: 20.0,
            luma_nr_detail: 41.0,
            luma_nr_contrast: 15.0,
            color_nr_amount: 30.0,
            color_nr_detail: 44.0,
            color_nr_smoothness: 56.0,
            capture_sharpen: !defaults.detail.capture_sharpen,
            capture_corner_boost: 0.3,
            ai_denoise: true,
            ai_denoise_strength: 0.37,
            ai_denoise_detail: 0.4,
        },
        effects: EffectsEdits {
            vignette_amount: 40.0,
            vignette_midpoint: 31.0,
            vignette_feather: 32.0,
            vignette_roundness: 12.0,
            grain_amount: 30.0,
            grain_size: 33.0,
            grain_roughness: 34.0,
        },
        lens: LensEdits {
            profile_enabled: Some(true),
            ca_enabled: true,
            constrain_crop: true,
            distortion_amount: 80.0,
            vignette_amount: 70.0,
            k1: 0.01,
            k2: 0.02,
            k3: 0.03,
            vk1: 0.04,
            vk2: 0.05,
            vk3: 0.06,
            ca_red_scale_x10000: 1.5,
            ca_blue_scale_x10000: -1.5,
        },
        geometry,
        masks: vec![serde_json::from_value(serde_json::json!({ "id": "layer" })).unwrap()],
        retouch: vec![stroke("incoming")],
        lens_blur: LensBlurEdits {
            amount: 60.0,
            focus_min: 0.1,
            focus_max: 0.6,
            shape: BokehShape::Hexagon,
            boost: 20.0,
            depth_raster_id: "theirs".to_string(),
        },
        unknown_ops,
    }
}

fn assert_every_field_differs<T: Serialize + Default>(label: &str, value: &T) {
    let value = serde_json::to_value(value).unwrap();
    let default = serde_json::to_value(T::default()).unwrap();
    for (key, field) in value.as_object().unwrap() {
        assert_ne!(
            Some(field),
            default.get(key),
            "{label}.{key} keeps its default"
        );
    }
}

fn only(set: impl FnOnce(&mut MergeSections)) -> MergeSections {
    let mut tools = MergeSections::default();
    set(&mut tools);
    tools
}

fn every_tool() -> MergeSections {
    MergeSections {
        geometry: true,
        masks: true,
        retouch: true,
        ..MergeSections::paste_default()
    }
}

fn legacy(set: impl FnOnce(&mut LegacySections)) -> LegacySections {
    let mut sections = LegacySections::default();
    set(&mut sections);
    sections
}

#[test]
fn the_fixture_sets_every_field() {
    let e = everything();
    assert_every_field_differs("basic", &e.basic);
    assert_every_field_differs("tone", &e.tone);
    assert_every_field_differs("color", &e.color);
    assert_every_field_differs("detail", &e.detail);
    assert_every_field_differs("effects", &e.effects);
    assert_every_field_differs("lens", &e.lens);
    assert_every_field_differs("lens_blur", &e.lens_blur);
    assert_every_field_differs("edits", &e);
}

#[test]
fn paste_default_applies_the_look_and_lens_blur_settings() {
    let mut current = Edits::default();
    current.geometry.rotate = 270;
    current.lens_blur.depth_raster_id = "mine".into();
    current
        .unknown_ops
        .insert("future_op".to_string(), serde_json::json!({ "x": 1 }));
    let merged = merge_edits(current, everything(), MergeSections::paste_default());
    assert_eq!(merged.effects.vignette_amount, 40.0);
    assert_eq!(merged.lens_blur.amount, 60.0);
    assert_eq!(merged.lens_blur.depth_raster_id, "mine");
    assert_eq!(merged.geometry.rotate, 270);
    assert_eq!(
        merged.unknown_ops["future_op"],
        serde_json::json!({ "x": 1 })
    );
}

#[test]
fn only_selected_sections_apply() {
    let merged = merge_edits(Edits::default(), everything(), only(|t| t.geometry = true));
    assert_eq!(merged.geometry.rotate, 90);
    assert_eq!(merged.effects.vignette_amount, 0.0);
}

#[test]
fn look_only_excludes_geometry_masks_and_retouch() {
    let current = Edits {
        retouch: vec![stroke("current")],
        ..Default::default()
    };
    let merged = merge_edits(current, everything(), MergeSections::look_only());
    assert_eq!(merged.effects.vignette_amount, 40.0);
    assert_eq!(merged.geometry.rotate, 0);
    assert!(merged.masks.is_empty());
    assert_eq!(merged.retouch, vec![stroke("current")]);
}

#[test]
fn a_depth_map_never_travels_to_another_photo() {
    let mut current = Edits::default();
    current.lens_blur.depth_raster_id = "mine".into();
    let merged = merge_edits(current, everything(), every_tool());
    assert_eq!(merged.lens_blur.depth_raster_id, "mine");
}

#[test]
fn lens_blur_settings_travel_without_the_depth_map() {
    let mut current = Edits::default();
    current.lens_blur.depth_raster_id = "mine".into();
    let merged = merge_edits(current, everything(), only(|t| t.lens_blur = true));
    assert_eq!(
        merged.lens_blur,
        LensBlurEdits {
            depth_raster_id: "mine".to_string(),
            ..everything().lens_blur
        }
    );
    assert_eq!(merged.effects, EffectsEdits::default());
}

#[test]
fn look_only_keeps_the_target_lens_blur() {
    let merged = merge_edits(Edits::default(), everything(), MergeSections::look_only());
    assert_eq!(merged.lens_blur, LensBlurEdits::default());
}

#[test]
fn hsl_alone_leaves_curves_and_bw() {
    let merged = merge_edits(Edits::default(), everything(), only(|t| t.hsl = true));
    assert_eq!(merged.color.hsl, everything().color.hsl);
    assert_eq!(merged.basic.curves, Edits::default().basic.curves);
    assert!(!merged.color.bw.enabled);
    assert_eq!(merged.basic.exposure_ev, 0.0);
}

#[test]
fn white_balance_alone_leaves_tone_and_presence() {
    let merged = merge_edits(
        Edits::default(),
        everything(),
        only(|t| t.white_balance = true),
    );
    assert_eq!(merged.basic.white_balance, everything().basic.white_balance);
    assert_eq!(merged.basic.exposure_ev, 0.0);
    assert_eq!(merged.basic.clarity, 0.0);
    assert_eq!(merged.tone.highlights, 0.0);
}

#[test]
fn tone_spans_basic_and_tone_regions() {
    let merged = merge_edits(Edits::default(), everything(), only(|t| t.tone = true));
    assert_eq!(merged.basic.exposure_ev, 1.0);
    assert_eq!(merged.tone, everything().tone);
    assert_eq!(merged.basic.clarity, 0.0);
}

#[test]
fn detail_and_effects_split_per_tool() {
    let merged = merge_edits(
        Edits::default(),
        everything(),
        only(|t| {
            t.noise_reduction = true;
            t.grain = true;
        }),
    );
    assert_eq!(merged.detail.luma_nr_amount, 20.0);
    assert!(merged.detail.ai_denoise);
    assert_eq!(merged.detail.sharpen_amount, None);
    assert_eq!(merged.effects.grain_amount, 30.0);
    assert_eq!(merged.effects.vignette_amount, 0.0);
}

#[test]
fn retouch_section_copies_strokes() {
    let current = Edits {
        retouch: vec![stroke("current")],
        ..Default::default()
    };
    let merged = merge_edits(current, everything(), only(|t| t.retouch = true));
    assert_eq!(merged.retouch, vec![stroke("incoming")]);
    assert_eq!(merged.geometry.rotate, 0);
}

#[test]
fn every_tool_copies_everything_but_the_depth_map() {
    let merged = merge_edits(Edits::default(), everything(), every_tool());
    let expected = Edits {
        lens_blur: LensBlurEdits {
            depth_raster_id: String::new(),
            ..everything().lens_blur
        },
        unknown_ops: Default::default(),
        ..everything()
    };
    assert_eq!(merged, expected);
}

#[test]
fn new_shape_reads_per_tool_keys() {
    let tools: MergeSections =
        serde_json::from_value(serde_json::json!({ "hsl": true, "lens_blur": true })).unwrap();
    assert_eq!(
        tools,
        only(|t| {
            t.hsl = true;
            t.lens_blur = true;
        })
    );
}

#[test]
fn a_legacy_basic_section_copies_the_whole_basic_struct_only() {
    let merged = merge_legacy(Edits::default(), everything(), legacy(|s| s.basic = true));
    assert_eq!(
        merged,
        Edits {
            basic: everything().basic,
            ..Edits::default()
        }
    );
}

#[test]
fn a_legacy_tone_section_copies_the_tone_struct_only() {
    let merged = merge_legacy(Edits::default(), everything(), legacy(|s| s.tone = true));
    assert_eq!(
        merged,
        Edits {
            tone: everything().tone,
            ..Edits::default()
        }
    );
}

#[test]
fn legacy_sections_keep_the_target_lens_blur_and_unknown_ops() {
    let all = LegacySections {
        basic: true,
        tone: true,
        color: true,
        detail: true,
        effects: true,
        lens: true,
        geometry: true,
        masks: true,
        retouch: true,
    };
    let mut current = Edits::default();
    current.lens_blur.depth_raster_id = "mine".into();
    let merged = merge_legacy(current.clone(), everything(), all);
    assert_eq!(
        merged,
        Edits {
            lens_blur: current.lens_blur,
            unknown_ops: current.unknown_ops,
            ..everything()
        }
    );
}
