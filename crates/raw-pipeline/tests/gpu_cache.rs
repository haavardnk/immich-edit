use std::sync::Arc;

use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
};
use raw_pipeline::frame::{OutputFormat, RawFrame, RenderOptions};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};

mod common;

use common::{haze_frame, rgb8_opts, synthetic_frame, try_renderer, try_renderer_with_budget};

fn nr_edits() -> Edits {
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 30.0;
    edits
}

#[test]
fn stage_textures_stay_inside_the_texture_budget() {
    const BUDGET: u64 = 256 * 1024;
    let Some(renderer) = try_renderer_with_budget(BUDGET) else {
        return;
    };
    let frames: Vec<RawFrame> = (0..6).map(|_| synthetic_frame(96, 64)).collect();
    let edits = nr_edits();
    let opts = rgb8_opts(96);
    for frame in &frames {
        renderer.render(frame, &edits, &opts).unwrap();
    }

    let stats = renderer.pool_stats();
    let budgeted = stats.texture_pool + stats.wb_cache + stats.nr_cache + stats.capture_cache;
    if budgeted > BUDGET {
        panic!("cached gpu textures used {budgeted} bytes for a {BUDGET} byte budget");
    }
    if stats.nr_cache == 0 {
        panic!("no noise reduction result was cached");
    }
}

#[test]
fn a_large_budget_keeps_more_than_two_stage_textures() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frames: Vec<RawFrame> = (0..4).map(|_| synthetic_frame(96, 64)).collect();
    let edits = nr_edits();
    let opts = rgb8_opts(96);

    renderer.render(&frames[0], &edits, &opts).unwrap();
    let one = renderer.pool_stats().nr_cache;
    for frame in &frames[1..] {
        renderer.render(frame, &edits, &opts).unwrap();
    }
    let all = renderer.pool_stats().nr_cache;

    if one == 0 {
        panic!("no noise reduction result was cached");
    }
    if all < one * 4 {
        panic!(
            "only {all} bytes of noise reduction results survived, expected {} ",
            one * 4
        );
    }
}

#[test]
fn atmosphere_is_reused_across_display_edits() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = haze_frame(96, 64);
    let opts = rgb8_opts(96);
    let render = |exposure_ev: f64, wb_temp: f64, luma_nr_amount: f64| {
        let mut edits = Edits::default();
        edits.basic.dehaze = 0.4;
        edits.basic.exposure_ev = exposure_ev;
        edits.basic.wb_temp = wb_temp;
        edits.detail.luma_nr_amount = luma_nr_amount;
        renderer.render(&frame, &edits, &opts).unwrap()
    };

    render(0.0, 0.0, 0.0);
    let after_first = renderer.atmosphere_estimates();
    render(0.7, 0.0, 0.0);
    render(-0.5, 60.0, 0.0);
    let after_rest = renderer.atmosphere_estimates();

    if after_rest != after_first {
        panic!(
            "exposure and white balance re-estimated the atmosphere: {after_first} -> {after_rest}"
        );
    }

    render(0.0, 0.0, 40.0);
    let after_nr = renderer.atmosphere_estimates();
    if after_nr == after_rest {
        panic!("a noise reduction change reused a stale atmosphere: {after_nr}");
    }
}

#[test]
fn a_white_balance_change_reuses_the_sensor_stage() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    let render = |wb_temp: f64| {
        let mut edits = nr_edits();
        edits.basic.wb_temp = wb_temp;
        renderer.render(&frame, &edits, &opts).unwrap()
    };

    let neutral = render(0.0);
    let cached = renderer.pool_stats();
    let warm = render(40.0);
    let after = renderer.pool_stats();

    if neutral.bytes == warm.bytes {
        panic!("the white balance change had no effect");
    }
    if (after.wb_cache, after.nr_cache) != (cached.wb_cache, cached.nr_cache) {
        panic!(
            "a white balance change re-ran the sensor stage: wb {} -> {}, nr {} -> {}",
            cached.wb_cache, after.wb_cache, cached.nr_cache, after.nr_cache
        );
    }
}

#[test]
fn display_textures_are_reused_across_ticks() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let (w, h) = (768, 512);
    let frame = haze_frame(w, h);
    let opts = rgb8_opts(w as u32);
    let mut edits = Edits::default();
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 25.0;
    edits.tone.shadows = 30.0;
    renderer.render(&frame, &edits, &opts).unwrap();
    let first = renderer.pool_stats().texture_pool;
    for exposure_ev in [0.2, 0.4, 0.6] {
        edits.basic.exposure_ev = exposure_ev;
        renderer.render(&frame, &edits, &opts).unwrap();
    }
    let later = renderer.pool_stats().texture_pool;

    let full_size = (w * h * 8) as u64;
    if first < 3 * full_size {
        panic!("only {first} bytes of display textures went back to the pool");
    }
    if later != first {
        panic!("display ticks grew the texture pool from {first} to {later} bytes");
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
