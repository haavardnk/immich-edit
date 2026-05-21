use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::immich::dto::SearchAssets;
use crate::state::AppState;

pub async fn metadata(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let assets = state.immich.search_metadata(&body).await?;
    Ok(Json(assets))
}
