use axum::Json;
use axum::extract::{Path, State};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::denoise::DenoiseStatus;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

pub async fn status(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Json<DenoiseStatus> {
    Json(
        state
            .denoise
            .status(RenderIdentity::from(&ctx), id.source())
            .await,
    )
}

pub async fn start(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<DenoiseStatus>, AppError> {
    let status = state
        .denoise
        .start(RenderIdentity::from(&ctx), &ctx.immich, id.source())
        .await?;
    Ok(Json(status))
}
