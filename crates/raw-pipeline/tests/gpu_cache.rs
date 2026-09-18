use raw_pipeline::edits::Edits;

mod common;

use common::{haze_frame, rgb8_opts, try_renderer};

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
