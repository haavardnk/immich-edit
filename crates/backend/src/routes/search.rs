use axum::Json;

use crate::error::AppError;
use crate::immich::dto::{SearchAssets, SearchStatistics};
use crate::routes::auth::AuthCtx;

pub async fn metadata(
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let assets = ctx.immich.search_metadata(&body).await?;
    Ok(Json(assets))
}

pub async fn smart(
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let assets = ctx.immich.search_smart(&body).await?;
    Ok(Json(assets))
}

pub async fn statistics(
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchStatistics>, AppError> {
    let stats = ctx.immich.search_statistics(&body).await?;
    Ok(Json(stats))
}
