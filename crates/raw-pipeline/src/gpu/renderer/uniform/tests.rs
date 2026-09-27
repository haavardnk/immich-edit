use super::super::UNIFORM_POOL_CAP_PER_SIZE;
use super::GpuRenderer;
use crate::edits::Edits;
use crate::frame::{FrameMeta, OutputFormat, RawFrame, RenderOptions};
use crate::gpu::passes::effects_tone::EffectsToneParams;
use crate::gpu::passes::sharpen::{SharpenBlurParams, SharpenParams};
use crate::gpu::uniform_pool::PooledUniform;

const W: usize = 96;
const H: usize = 64;

fn frame() -> RawFrame {
    let data = (0..W * H)
        .flat_map(|i| {
            let x = i % W;
            let y = i / W;
            let checker = if (x / 3 + y / 3) % 2 == 0 { 0.7 } else { 0.2 };
            [checker, x as f32 / W as f32, y as f32 / H as f32]
        })
        .collect();
    RawFrame {
        meta: FrameMeta {
            width: W,
            height: H,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data,
        cpp: 3,
        exif: None,
    }
}

fn contend(renderer: &GpuRenderer) -> Vec<PooledUniform> {
    let sizes = [
        size_of::<SharpenBlurParams>(),
        size_of::<SharpenParams>(),
        size_of::<EffectsToneParams>(),
    ];
    sizes
        .iter()
        .flat_map(|&size| std::iter::repeat_n(size, UNIFORM_POOL_CAP_PER_SIZE))
        .map(|size| {
            renderer.uniform_pool.acquire(
                &renderer.ctx.device,
                &renderer.ctx.queue,
                &vec![0x3f; size],
                "contender",
            )
        })
        .collect()
}

fn render(renderer: &GpuRenderer, raw: &RawFrame, edits: &Edits, contended: bool) -> Vec<u8> {
    let opts = RenderOptions {
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    };
    let source = match renderer.linear_source(raw, edits, &opts) {
        Ok(source) => source,
        Err(e) => panic!("linear source: {e}"),
    };
    let display = match renderer.render_display(&source, edits, &opts) {
        Ok(display) => display,
        Err(e) => panic!("render display: {e}"),
    };
    let held = if contended {
        contend(renderer)
    } else {
        Vec::new()
    };
    let image = renderer
        .read_display(display, None)
        .and_then(|readback| readback.into_rendered(&opts, false, None));
    drop(held);
    match image {
        Ok(image) => image.bytes,
        Err(e) => panic!("read display: {e}"),
    }
}

#[test]
fn display_uniforms_survive_pool_reuse_before_submit() {
    let Ok(renderer) = GpuRenderer::new() else {
        eprintln!("no gpu adapter, skipping");
        return;
    };
    let raw = frame();
    let mut edits = Edits::default();
    edits.detail.sharpen_amount = Some(80.0);
    edits.effects.vignette_amount = -40.0;
    let clean = render(&renderer, &raw, &edits, false);
    let contended = render(&renderer, &raw, &edits, true);
    if clean != contended {
        let changed = clean.iter().zip(&contended).filter(|(a, b)| a != b).count();
        panic!(
            "{changed} of {} bytes changed when another render reused the pooled uniforms",
            clean.len()
        );
    }
}
