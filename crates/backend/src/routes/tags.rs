use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{BulkIdResponse, TagSummary};
use crate::routes::auth::AuthCtx;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<TagSummary>>, AppError> {
    let mut tags = ctx.immich.list_tags().await?;
    let ids: Vec<Uuid> = tags.iter().map(|t| t.id).collect();
    let counts = state.tag_counts.counts_for(&ctx.immich, &ids).await;
    for tag in &mut tags {
        tag.asset_count = counts.get(&tag.id).copied();
    }
    Ok(Json(tags))
}

pub async fn upsert(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Vec<TagSummary>>, AppError> {
    let tags = ctx.immich.upsert_tags(&body).await?;
    state.tag_counts.clear().await;
    Ok(Json(tags))
}

pub async fn tag_asset(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((tag_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = ctx.immich.tag_asset(tag_id, asset_id).await?;
    state.tag_counts.invalidate(tag_id).await;
    Ok(Json(resp))
}

pub async fn untag_asset(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((tag_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = ctx.immich.untag_asset(tag_id, asset_id).await?;
    state.tag_counts.invalidate(tag_id).await;
    Ok(Json(resp))
}
