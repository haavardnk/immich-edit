use raw_pipeline::edits::Edits;
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
