use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::immich::dto::{SearchAssets, SearchStatistics};
use crate::state::AppState;

pub async fn metadata(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let assets = state.immich.search_metadata(&body).await?;
    Ok(Json(assets))
}

pub async fn statistics(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchStatistics>, AppError> {
    let stats = state.immich.search_statistics(&body).await?;
    Ok(Json(stats))
}
