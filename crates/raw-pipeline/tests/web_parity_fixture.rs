use std::path::Path;
use std::sync::Arc;

use raw_pipeline::edits::{CropRect, Edits};
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use raw_pipeline::lut::CubeLut;
use raw_pipeline::mask_raster::RasterMap;
use raw_pipeline_testkit::depth::{DEPTH_ID, depth_raster};
use raw_pipeline_testkit::fixtures::fixture_path;
use raw_pipeline_testkit::gpu::try_renderer;

const FIXTURE: &str = "Canon_EOS_5D_3-2.cr2";
const DCP: &str = "Canon EOS 5D.dcp";
const LUT_ID: &str = "warm";
const DEPTH_DIMS: (u32, u32) = (192, 128);
const CUBE: &str =
    "LUT_3D_SIZE 2\n0 0 0\n1 0.05 0\n0 0.95 0\n1 1 0\n0 0 0.9\n1 0.05 0.9\n0 0.95 0.9\n1 1 0.9\n";

fn edits(lens_blur: bool) -> serde_json::Value {
    let mut edits = base_edits();
    if lens_blur {
        edits["lens_blur"] = serde_json::json!({
            "amount": 80.0,
            "focus_min": 0.0,
            "focus_max": 0.15,
            "shape": "hexagon",
            "boost": 30.0,
            "depth_raster_id": DEPTH_ID
        });
    }
    edits
}

fn base_edits() -> serde_json::Value {
    serde_json::json!({
        "basic": {
            "exposure_ev": 0.35,
            "contrast": 18.0,
            "white_balance": {"mode": "relative", "temp": 12.0, "tint": 0.0},
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
    lens_blur: bool,
}

const CASES: [Case; 3] = [
    Case {
        name: "fit",
        max_edge: 384,
        roi: None,
        lens_blur: false,
    },
    Case {
        name: "tile",
        max_edge: 256,
        roi: Some([0.5, 0.3, 0.25, 0.25]),
        lens_blur: false,
    },
    Case {
        name: "lens_blur",
        max_edge: 384,
        roi: None,
        lens_blur: true,
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
    let out = root.join("web/e2e/fixtures/render");
    std::fs::write(out.join("lut.cube"), CUBE).unwrap();
    let depth = Arc::new(depth_raster(DEPTH_DIMS.0, DEPTH_DIMS.1));
    for case in CASES {
        let edits_json = edits(case.lens_blur);
        let edits: Edits = serde_json::from_value(edits_json.clone()).unwrap();
        let rasters: RasterMap = if case.lens_blur {
            RasterMap::from([(DEPTH_ID.to_string(), depth.clone())])
        } else {
            RasterMap::new()
        };
        let opts = RenderOptions {
            rasters,
            max_edge: case.max_edge,
            output: OutputFormat::Rgb8,
            histogram: true,
            roi: case.roi.map(|[x, y, w, h]| CropRect { x, y, w, h }),
            dcp: Some(Arc::new(raw_pipeline::parse_dcp(&dcp_bytes).unwrap())),
            luts: [(
                LUT_ID.to_string(),
                Arc::new(CubeLut::parse(CUBE.as_bytes()).unwrap()),
            )]
            .into(),
            ..Default::default()
        };
        let image = renderer
            .render_source(&frame, &edits, &opts, None)
            .unwrap()
            .image;
        if case.roi.is_some() != image.header.window.is_some()
            || case.roi.is_some() != image.header.context.is_some()
        {
            panic!("{}: the source window does not follow the roi", case.name);
        }
        let rendered = renderer.render(&frame, &edits, &opts).unwrap();
        let dir = out.join(case.name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("source.iesr"),
            raw_pipeline::source::encode(&image, raw_pipeline::source::SourceCoding::Framed)
                .unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("expected.rgb"), &rendered.bytes).unwrap();
        let rasters = if case.lens_blur {
            std::fs::write(dir.join("depth.r8"), &depth.bytes).unwrap();
            serde_json::json!([{
                "id": DEPTH_ID,
                "file": "depth.r8",
                "width": DEPTH_DIMS.0,
                "height": DEPTH_DIMS.1
            }])
        } else {
            serde_json::json!([])
        };
        let spec = serde_json::json!({
            "dcp": DCP,
            "lut_id": LUT_ID,
            "rasters": rasters,
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
