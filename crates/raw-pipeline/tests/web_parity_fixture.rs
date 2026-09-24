mod common;

use std::path::Path;
use std::sync::Arc;

use common::{fixture_path, try_renderer};
use raw_pipeline::edits::{CropRect, Edits};
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use raw_pipeline::lut::Lut3d;

const FIXTURE: &str = "Canon_EOS_5D_3-2.cr2";
const DCP: &str = "Canon EOS 5D.dcp";
const LUT_ID: &str = "warm";
const CUBE: &str =
    "LUT_3D_SIZE 2\n0 0 0\n1 0.05 0\n0 0.95 0\n1 1 0\n0 0 0.9\n1 0.05 0.9\n0 0.95 0.9\n1 1 0.9\n";

fn edits() -> serde_json::Value {
    serde_json::json!({
        "basic": {
            "exposure_ev": 0.35,
            "contrast": 18.0,
            "wb_temp": 12.0,
            "texture": 25.0,
            "clarity": 15.0,
            "dehaze": 20.0,
            "vibrance": 10.0
        },
        "tone": {"shadows": 35.0, "highlights": -40.0},
        "color": {
            "hsl": {"bands": [
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 20.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0},
                {"hue": 0.0, "sat": 0.0, "lum": 0.0}
            ]},
            "lut_3d": {"lut_id": LUT_ID, "amount": 60.0},
            "dcp": {"mode": "profile", "profile_id": "bundled"}
        },
        "detail": {"sharpen_amount": 50.0, "luma_nr_amount": 25.0},
        "effects": {"vignette_amount": -25.0},
        "masks": [{
            "id": "sky",
            "name": "",
            "enabled": true,
            "color": "#3b82f6",
            "amount": 1.0,
            "invert": false,
            "components": [{
                "id": "c1",
                "enabled": true,
                "mode": "add",
                "invert": false,
                "kind": {"kind": "linear", "p0": {"x": 0.5, "y": 0.0}, "p1": {"x": 0.5, "y": 0.6}, "feather": 0.5},
                "source": "manual"
            }],
            "edits": {"exposure_ev": -0.4, "wb_temp": -15.0, "clarity": 20.0}
        }]
    })
}

struct Case {
    name: &'static str,
    max_edge: u32,
    roi: Option<[f32; 4]>,
}

const CASES: [Case; 2] = [
    Case {
        name: "fit",
        max_edge: 384,
        roi: None,
    },
    Case {
        name: "tile",
        max_edge: 256,
        roi: Some([0.5, 0.3, 0.25, 0.25]),
    },
];

#[test]
fn bake_web_parity_fixture() {
    if std::env::var("BAKE_WEB_PARITY").is_err() {
        return;
    }
    let renderer = try_renderer().expect("baking the web parity fixture needs a gpu");
    let frame =
        raw_pipeline::decode::decode(&std::fs::read(fixture_path(FIXTURE)).unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dcp_bytes = std::fs::read(root.join("crates/backend/assets/dcp").join(DCP)).unwrap();
    let edits_json = edits();
    let edits: Edits = serde_json::from_value(edits_json.clone()).unwrap();
    let out = root.join("web/e2e/fixtures/render");
    std::fs::write(out.join("lut.cube"), CUBE).unwrap();
    for case in CASES {
        let opts = RenderOptions {
            max_edge: case.max_edge,
            output: OutputFormat::Rgb8,
            histogram: true,
            roi: case.roi.map(|[x, y, w, h]| CropRect { x, y, w, h }),
            dcp: Some(Arc::new(raw_pipeline::parse_dcp(&dcp_bytes).unwrap())),
            luts: [(
                LUT_ID.to_string(),
                Arc::new(Lut3d::parse_cube(CUBE.as_bytes()).unwrap()),
            )]
            .into(),
            ..Default::default()
        };
        let image = renderer
            .render_source(&frame, &edits.sensor_stage(), &opts, None)
            .unwrap()
            .image;
        if case.roi.is_some() != image.header.window.is_some() {
            panic!("{}: the source window does not follow the roi", case.name);
        }
        let rendered = renderer.render(&frame, &edits, &opts).unwrap();
        let dir = out.join(case.name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("source.iesr"),
            raw_pipeline::source::encode(&image).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("expected.rgb"), &rendered.bytes).unwrap();
        let spec = serde_json::json!({
            "dcp": DCP,
            "lut_id": LUT_ID,
            "edits": edits_json,
            "view": {
                "max_edge": case.max_edge,
                "histogram": true,
                "roi": case.roi,
                "tile": case.roi.is_some()
            },
            "width": rendered.width,
            "height": rendered.height,
            "adapter": renderer.adapter_label()
        });
        std::fs::write(
            dir.join("case.json"),
            serde_json::to_string_pretty(&spec).unwrap() + "\n",
        )
        .unwrap();
    }
}
