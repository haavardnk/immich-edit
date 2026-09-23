use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RenderOptions, RenderedImage};
use raw_pipeline::{CpuRenderer, GpuRenderer};

mod common;

use common::{mean_abs_delta, rgb8_opts, synthetic_bayer_frame};

const W: usize = 512;
const H: usize = 384;

fn require_geometry(label: &str, image: &RenderedImage, out: (u32, u32)) {
    let got = (image.width, image.height, image.source_w, image.source_h);
    let want = (out.0, out.1, W as u32, H as u32);
    if got != want {
        panic!("{label}: output and source dims {got:?}, want {want:?}");
    }
}

#[test]
fn superpixel_preview_keeps_full_geometry_and_matches_full_demosaic() {
    let frame = synthetic_bayer_frame(W, H, "RGGB");
    let edits = Edits::default();
    let cpu = CpuRenderer::new();
    let preview = cpu.render(&frame, &edits, &rgb8_opts(128)).unwrap();
    let full = cpu
        .render(
            &frame,
            &edits,
            &RenderOptions {
                quality: true,
                ..rgb8_opts(128)
            },
        )
        .unwrap();
    require_geometry("cpu preview", &preview, (128, 96));
    require_geometry("cpu full", &full, (128, 96));
    let delta = mean_abs_delta(&preview.bytes, &full.bytes);
    if delta > 3.0 {
        panic!("superpixel preview drifts {delta:.2}/255 from the full demosaic");
    }
}

#[test]
fn gpu_superpixel_matches_cpu_and_coexists_with_the_full_frame() {
    let renderer = match GpuRenderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            return;
        }
    };
    let frame = synthetic_bayer_frame(W, H, "RGGB");
    let edits = Edits::default();
    let export = RenderOptions {
        quality: true,
        ..rgb8_opts(W as u32)
    };
    let full_before = renderer.render(&frame, &edits, &export).unwrap();
    let gpu = renderer.render(&frame, &edits, &rgb8_opts(128)).unwrap();
    let full_after = renderer.render(&frame, &edits, &export).unwrap();
    let cpu = CpuRenderer::new()
        .render(&frame, &edits, &rgb8_opts(128))
        .unwrap();

    require_geometry("gpu preview", &gpu, (128, 96));
    require_geometry("gpu export", &full_after, (W as u32, H as u32));
    let delta = mean_abs_delta(&cpu.bytes, &gpu.bytes);
    if delta > 1.0 {
        panic!("gpu superpixel preview drifts {delta:.2}/255 from cpu");
    }
    if full_before.bytes != full_after.bytes {
        panic!("the export changed after a superpixel preview shared the frame cache");
    }
}
