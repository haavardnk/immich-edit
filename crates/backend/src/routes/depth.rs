use axum::Json;
use axum::extract::{Path, State};
use ml::ModelKind;
use serde::Serialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::masks::{map_segment_err, scene_render};
use crate::services::mask_scene::SceneImage;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DepthResponse {
    pub raster_id: String,
    pub width: u32,
    pub height: u32,
    pub model_id: String,
    pub backend: &'static str,
    pub elapsed_ms: u64,
}

pub async fn generate(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(asset_id): Path<AssetKey>,
) -> Result<Json<DepthResponse>, AppError> {
    state
        .segment
        .ready(ModelKind::Depth)
        .await
        .map_err(map_segment_err)?;
    let SceneImage {
        rgb8,
        width,
        height,
    } = scene_render(&state, &ctx, asset_id).await?;
    let run = state
        .segment
        .depth(rgb8, width, height)
        .await
        .map_err(map_segment_err)?;
    let stored = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &run.output, width, height)
        .await?;
    Ok(Json(DepthResponse {
        raster_id: stored.raster_id,
        width,
        height,
        model_id: run.model_id,
        backend: run.backend,
        elapsed_ms: run.elapsed_ms,
    }))
}
