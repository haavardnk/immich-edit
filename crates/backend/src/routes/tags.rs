use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::immich::dto::TagSummary;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TagSummary>>, AppError> {
    let tags = state.immich.list_tags().await?;
    Ok(Json(tags))
}
