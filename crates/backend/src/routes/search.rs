use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::immich::dto::{SearchAssets, SearchStatistics};
use crate::routes::auth::AuthCtx;
use crate::services::copy_expand::expand_assets;
use crate::state::AppState;

pub async fn metadata(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let mut assets = ctx.immich.search_metadata(&body).await?;
    assets.items = expand_assets(&state.edits, ctx.owner, assets.items).await?;
    Ok(Json(assets))
}

pub async fn smart(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let mut assets = ctx.immich.search_smart(&body).await?;
    assets.items = expand_assets(&state.edits, ctx.owner, assets.items).await?;
    Ok(Json(assets))
}

pub async fn statistics(
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchStatistics>, AppError> {
    let stats = ctx.immich.search_statistics(&body).await?;
    Ok(Json(stats))
}
