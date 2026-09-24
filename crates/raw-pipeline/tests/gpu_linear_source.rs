mod common;

use common::{haze_frame, rgb8_opts, try_renderer};
use raw_pipeline::PipelineError;
use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    Vec2f,
};
use raw_pipeline::source::LinearKind;

fn wb_layer() -> MaskLayer {
    MaskLayer {
        id: "warm".into(),
        name: String::new(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Linear {
                p0: Vec2f { x: 0.0, y: 0.5 },
                p1: Vec2f { x: 1.0, y: 0.5 },
                feather: 0.4,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            wb_temp: Some(20.0),
            texture: Some(30.0),
            ..Default::default()
        },
    }
}

fn case(label: &str) -> Edits {
    let mut edits = Edits::default();
    match label {
        "fast" => edits.basic.exposure_ev = 0.4,
        "presence" => {
            edits.basic.dehaze = 0.3;
            edits.basic.texture = 25.0;
            edits.tone.shadows = 40.0;
        }
        "noise" => {
            edits.detail.luma_nr_amount = 40.0;
            edits.detail.color_nr_amount = 30.0;
        }
        "layer-wb" => {
            edits.basic.dehaze = 0.2;
            edits.masks = vec![wb_layer()];
        }
        other => panic!("unknown case {other}"),
    }
    edits
}

#[test]
fn a_linear_source_renders_the_same_bytes_as_its_raw_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = haze_frame(96, 64);
    let opts = rgb8_opts(96);
    for (label, kind) in [
        ("fast", LinearKind::PreWb),
        ("presence", LinearKind::PostWb),
        ("noise", LinearKind::PostWb),
        ("layer-wb", LinearKind::PostWb),
    ] {
        let source = renderer.linear_source(&frame, &case(label), &opts).unwrap();
        if source.kind != kind {
            panic!("{label}: source is {:?}, expected {kind:?}", source.kind);
        }
        for wb_temp in [0.0, 35.0] {
            let mut edits = case(label);
            edits.basic.wb_temp = wb_temp;
            let from_raw = renderer.render(&frame, &edits, &opts).unwrap();
            let from_source = renderer.render(&source, &edits, &opts).unwrap();
            if from_raw.bytes != from_source.bytes {
                panic!(
                    "{label} at wb_temp {wb_temp}: rendering the linear source changed the output"
                );
            }
        }
    }
}

#[test]
fn a_linear_source_rejects_edits_it_was_not_prepared_for() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = haze_frame(96, 64);
    let opts = rgb8_opts(96);
    for (prepared, rendered) in [("fast", "presence"), ("noise", "layer-wb")] {
        let source = renderer
            .linear_source(&frame, &case(prepared), &opts)
            .unwrap();
        match renderer.render(&source, &case(rendered), &opts) {
            Err(PipelineError::Unsupported(_)) => {}
            Err(e) => panic!("{prepared} source rendering {rendered}: unexpected error {e}"),
            Ok(_) => panic!("{prepared} source rendered {rendered} edits it cannot hold"),
        }
    }
}
