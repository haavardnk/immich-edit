use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{BulkIdResponse, TagSummary};
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TagSummary>>, AppError> {
    let tags = state.immich.list_tags().await?;
    Ok(Json(tags))
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Vec<TagSummary>>, AppError> {
    let tags = state.immich.upsert_tags(&body).await?;
    Ok(Json(tags))
}

pub async fn tag_asset(
    State(state): State<AppState>,
    Path((tag_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = state.immich.tag_asset(tag_id, asset_id).await?;
    Ok(Json(resp))
}

pub async fn untag_asset(
    State(state): State<AppState>,
    Path((tag_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = state.immich.untag_asset(tag_id, asset_id).await?;
    Ok(Json(resp))
}
