use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::services::render_telemetry::TelemetrySnapshot;
use crate::state::AppState;

#[derive(Serialize)]
pub struct GpuPoolBytes {
    pub texture_pool: u64,
    pub uniform_pool: u64,
    pub output_targets: u64,
    pub sharpen_targets: u64,
    pub wb_cache: u64,
    pub nr_cache: u64,
    pub atlas_cache: u64,
    pub total: u64,
}

#[derive(Serialize)]
pub struct Timings {
    pub renderer_active: &'static str,
    pub render_latency: TelemetrySnapshot,
    pub gpu_pool_bytes: Option<GpuPoolBytes>,
}

pub async fn timings(State(state): State<AppState>) -> Json<Timings> {
    let snapshot = state.render.telemetry().snapshot();
    let gpu_pool_bytes = state.render.gpu_pool_stats().map(|s| GpuPoolBytes {
        texture_pool: s.texture_pool,
        uniform_pool: s.uniform_pool,
        output_targets: s.output_targets,
        sharpen_targets: s.sharpen_targets,
        wb_cache: s.wb_cache,
        nr_cache: s.nr_cache,
        atlas_cache: s.atlas_cache,
        total: s.texture_pool
            + s.uniform_pool
            + s.output_targets
            + s.sharpen_targets
            + s.wb_cache
            + s.nr_cache
            + s.atlas_cache,
    });
    Json(Timings {
        renderer_active: state.render.active().as_str(),
        render_latency: snapshot,
        gpu_pool_bytes,
    })
}
