use raw_pipeline::GpuRenderer;
use raw_pipeline::GpuRendererOptions;

pub fn try_renderer() -> Option<GpuRenderer> {
    try_renderer_with(GpuRendererOptions::default())
}

pub fn try_renderer_with(options: GpuRendererOptions) -> Option<GpuRenderer> {
    match GpuRenderer::with_options(options) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            None
        }
    }
}
