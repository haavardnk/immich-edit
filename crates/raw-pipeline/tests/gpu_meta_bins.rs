use raw_pipeline::edits::Edits;
use raw_pipeline::frame::PreviewMode;
use raw_pipeline::histogram::Histogram;
use raw_pipeline::scopes::ScopeGrids;

mod common;

#[test]
fn gpu_display_bins_match_the_cpu_counters_on_every_fixture() {
    let Some(renderer) = common::try_renderer() else {
        return;
    };
    let mut edits = Edits::default();
    edits.basic.exposure_ev = 0.4;
    edits.basic.contrast = 20.0;
    edits.effects.vignette_amount = -30.0;
    common::each_fixture_frame(|name, frame| {
        for max_edge in [600, 1600] {
            let options = raw_pipeline::frame::RenderOptions {
                histogram: true,
                scopes: true,
                ..common::rgb8_opts(max_edge)
            };
            let image = match renderer.render(frame, &edits, &options) {
                Ok(image) => image,
                Err(e) => panic!("{name} @ {max_edge}: render failed: {e}"),
            };
            let w = image.width as usize;
            let h = image.height as usize;
            if image.histogram != Some(Histogram::from_rgb_u8(&image.bytes, w, h)) {
                panic!("{name} @ {max_edge}: display histogram differs from the CPU counter");
            }
            if image.scopes != Some(ScopeGrids::from_rgb_u8(&image.bytes, w, h)) {
                panic!("{name} @ {max_edge}: scope grids differ from the CPU counter");
            }
            if image.linear_histogram.is_none() {
                panic!("{name} @ {max_edge}: linear histogram missing");
            }
        }
    });
}

#[test]
fn a_lut_graded_frame_feeds_the_bins_and_the_mask_overlay() {
    let Some(renderer) = common::try_renderer() else {
        return;
    };
    let cube =
        "LUT_3D_SIZE 2\n0 0 0\n1 0.1 0\n0 0.9 0\n1 1 0\n0 0 0.8\n1 0.1 0.8\n0 0.9 0.8\n1 1 0.8\n";
    let edits: Edits = serde_json::from_value(serde_json::json!({
        "color": {"lut_3d": {"lut_id": "warm", "amount": 60.0}},
        "masks": [{
            "id": "top",
            "components": [{
                "id": "c1",
                "kind": {"kind": "linear", "p0": {"x": 0.5, "y": 0.0}, "p1": {"x": 0.5, "y": 0.6}}
            }],
            "edits": {"exposure_ev": -0.5}
        }]
    }))
    .unwrap();
    let luts: raw_pipeline::lut::LutMap = [(
        "warm".to_string(),
        std::sync::Arc::new(raw_pipeline::lut::Lut3d::parse_cube(cube.as_bytes()).unwrap()),
    )]
    .into();
    let frame = common::synthetic_frame(96, 64);
    for preview_mode in [
        PreviewMode::None,
        PreviewMode::MaskWeight {
            layer_id: "top".into(),
        },
    ] {
        let options = raw_pipeline::frame::RenderOptions {
            histogram: true,
            scopes: true,
            preview_mode: preview_mode.clone(),
            luts: luts.clone(),
            ..common::rgb8_opts(96)
        };
        let image = match renderer.render(&frame, &edits, &options) {
            Ok(image) => image,
            Err(e) => panic!("{preview_mode:?}: render failed: {e}"),
        };
        let w = image.width as usize;
        let h = image.height as usize;
        if image.histogram != Some(Histogram::from_rgb_u8(&image.bytes, w, h)) {
            panic!("{preview_mode:?}: display histogram differs from the CPU counter");
        }
    }
}
