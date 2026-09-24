mod common;

use common::{haze_frame, rgb8_opts, try_renderer};
use raw_pipeline::PipelineError;
use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    Vec2f,
};
use raw_pipeline::gpu::LinearKind;

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
    for (label, kind, layer_bases) in [
        ("fast", LinearKind::PreWb, 0),
        ("presence", LinearKind::PostWb, 0),
        ("noise", LinearKind::PostWb, 0),
        ("layer-wb", LinearKind::PostWb, 1),
    ] {
        let edits = case(label);
        let source = renderer.linear_source(&frame, &edits, &opts).unwrap();
        if source.kind != kind || source.layer_bases.len() != layer_bases {
            panic!(
                "{label}: source is {:?} with {} layer bases, expected {kind:?} with {layer_bases}",
                source.kind,
                source.layer_bases.len()
            );
        }
        let from_raw = renderer.render(&frame, &edits, &opts).unwrap();
        let from_source = renderer.render(&source, &edits, &opts).unwrap();
        if from_raw.bytes != from_source.bytes {
            panic!("{label}: rendering the linear source changed the output");
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
