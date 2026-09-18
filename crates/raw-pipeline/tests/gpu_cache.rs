use std::sync::Arc;

use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
};
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};

mod common;

use common::{haze_frame, rgb8_opts, synthetic_frame, try_renderer};

#[test]
fn atmosphere_is_reused_across_non_spatial_edits() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = haze_frame(96, 64);
    let opts = rgb8_opts(96);
    let render = |exposure_ev: f64, wb_temp: f64| {
        let mut edits = Edits::default();
        edits.basic.dehaze = 0.4;
        edits.basic.exposure_ev = exposure_ev;
        edits.basic.wb_temp = wb_temp;
        renderer.render(&frame, &edits, &opts).unwrap()
    };

    render(0.0, 0.0);
    let after_first = renderer.atmosphere_estimates();
    render(0.7, 0.0);
    render(-0.5, 0.0);
    let after_rest = renderer.atmosphere_estimates();

    if after_rest != after_first {
        panic!("exposure changes re-estimated the atmosphere: {after_first} -> {after_rest}");
    }

    render(0.0, 60.0);
    let after_wb = renderer.atmosphere_estimates();
    if after_wb == after_rest {
        panic!("a white balance change reused a stale atmosphere: {after_wb}");
    }
}

#[test]
fn mask_atlas_is_allocated_and_uploaded_once() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let w: u32 = 32;
    let h: u32 = 32;
    let mut bytes = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in w / 2..w {
            bytes[(y * w + x) as usize] = 255;
        }
    }
    let mut rasters = RasterMap::new();
    rasters.insert(
        "brush_a".into(),
        Arc::new(MaskRaster::new(w, h, bytes).unwrap()),
    );
    let opts = RenderOptions {
        max_edge: 96,
        output: OutputFormat::Rgb8,
        rasters,
        ..Default::default()
    };
    let brushed = |exposure_ev: f64| Edits {
        masks: vec![MaskLayer {
            id: "L1".into(),
            name: String::new(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            invert: false,
            components: vec![MaskComponent {
                id: "b1".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                invert: false,
                kind: MaskComponentKind::Brush {
                    raster_id: "brush_a".into(),
                },
                source: MaskSource::Manual,
                generated: None,
            }],
            edits: MaskedEdits {
                exposure_ev: Some(exposure_ev),
                ..Default::default()
            },
        }],
        ..Default::default()
    };

    let first = renderer.render(&frame, &brushed(1.0), &opts).unwrap();
    let second = renderer.render(&frame, &brushed(0.4), &opts).unwrap();

    if renderer.mask_atlas_allocations() != 1 {
        panic!(
            "mask atlas was allocated {} times",
            renderer.mask_atlas_allocations()
        );
    }
    if renderer.mask_atlas_uploads() != 1 {
        panic!(
            "brush raster was uploaded {} times",
            renderer.mask_atlas_uploads()
        );
    }
    if first.bytes == second.bytes {
        panic!("the two renders were identical, so the mask was not applied");
    }
}
