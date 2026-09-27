use raw_pipeline::{
    cpu, decode,
    edits::{BasicEdits, DetailEdits, Edits, ToneEdits},
    frame::RenderOptions,
};
use raw_pipeline_testkit::baseline::{RenderStats, Tolerance};
use raw_pipeline_testkit::fixtures::{baseline_path, fixture_path};
use raw_pipeline_testkit::render::decode_jpeg_rgb;
use std::collections::BTreeMap;

const FIXTURE: &str = "Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw";
const BASELINE: &str = "multi_feature.json";
const MAX_EDGE: u32 = 512;
const TOLERANCE: Tolerance = Tolerance {
    mean_rgb: 1.5,
    grid_cell: 4.0,
    grid_mean: 1.5,
};

type BaselineMap = BTreeMap<String, RenderStats>;

fn stack_tone_lift() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.25,
            contrast: 12.0,
            saturation: 4.0,
            vibrance: 18.0,
            wb_temp: 8.0,
            wb_tint: 3.0,
            clarity: -8.0,
            texture: -5.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -25.0,
            shadows: 30.0,
            blacks: 8.0,
            whites: -10.0,
        },
        detail: DetailEdits {
            sharpen_amount: Some(35.0),
            sharpen_masking: 45.0,
            color_nr_amount: 25.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn stack_clarity_dehaze() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: -0.1,
            contrast: 22.0,
            saturation: 8.0,
            vibrance: 30.0,
            wb_temp: -6.0,
            clarity: 25.0,
            texture: 15.0,
            dehaze: 12.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -35.0,
            shadows: 18.0,
            blacks: -5.0,
            whites: 8.0,
        },
        detail: DetailEdits {
            sharpen_amount: Some(60.0),
            sharpen_radius: 1.0,
            sharpen_detail: 35.0,
            sharpen_masking: 20.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn stack_noise_reduction() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.6,
            contrast: 18.0,
            saturation: -10.0,
            vibrance: 15.0,
            wb_temp: 4.0,
            clarity: 10.0,
            dehaze: 8.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -15.0,
            shadows: 40.0,
            blacks: 12.0,
            whites: -5.0,
        },
        detail: DetailEdits {
            sharpen_amount: Some(25.0),
            luma_nr_amount: 55.0,
            luma_nr_detail: 40.0,
            color_nr_amount: 60.0,
            color_nr_smoothness: 60.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn render_stack(edits: &Edits) -> RenderStats {
    let bytes = std::fs::read(fixture_path(FIXTURE)).expect("fixture missing");
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: MAX_EDGE,
        ..Default::default()
    };
    let out = cpu::render(&frame, edits, &opts).unwrap();
    let (rgb, w, h) = decode_jpeg_rgb(&out.bytes);
    RenderStats::measure(&rgb, w, h)
}

#[test]
fn multi_feature_stacks() {
    if !fixture_path(FIXTURE).exists() {
        eprintln!("skip: {FIXTURE} missing");
        return;
    }
    let stacks: [(&str, Edits); 3] = [
        ("tone_lift", stack_tone_lift()),
        ("clarity_dehaze", stack_clarity_dehaze()),
        ("noise_reduction", stack_noise_reduction()),
    ];
    let measured: BaselineMap = stacks
        .iter()
        .map(|(name, e)| ((*name).to_string(), render_stack(e)))
        .collect();

    if std::env::var_os("BAKE_MULTI_FEATURE").is_some() {
        let json = serde_json::to_string_pretty(&measured).unwrap();
        std::fs::write(baseline_path(BASELINE), json).unwrap();
        eprintln!("baked multi-feature baseline ({} stacks)", measured.len());
        return;
    }

    let raw = std::fs::read_to_string(baseline_path(BASELINE))
        .expect("multi_feature.json missing — run BAKE_MULTI_FEATURE=1");
    let baseline: BaselineMap = serde_json::from_str(&raw).unwrap();
    let failures: Vec<String> = measured
        .iter()
        .flat_map(|(name, got)| {
            let want = baseline
                .get(name)
                .unwrap_or_else(|| panic!("baseline missing stack {name}"));
            got.drift(want, &TOLERANCE)
                .into_iter()
                .map(move |e| format!("{name}: {e}"))
        })
        .collect();
    if !failures.is_empty() {
        panic!("multi-feature regressions:\n  {}", failures.join("\n  "));
    }
}
