use raw_pipeline::GpuRenderer;
use raw_pipeline::GpuRendererOptions;

pub fn try_renderer() -> Option<GpuRenderer> {
    match GpuRenderer::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            None
        }
    }
}

pub fn try_renderer_with_budget(texture_cache_max_bytes: u64) -> Option<GpuRenderer> {
    match GpuRenderer::with_options(GpuRendererOptions {
        texture_cache_max_bytes,
        timestamps: false,
    }) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            None
        }
    }
}
