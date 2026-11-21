use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::{BulkIdResponse, TagSummary};
use crate::routes::auth::AuthCtx;
use crate::routes::search::searcher;
use crate::state::AppState;

mod manage;

pub use manage::{create, delete, update};

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<TagSummary>>, AppError> {
    let mut tags = ctx.immich.list_tags().await?;
    let ids: Vec<Uuid> = tags.iter().map(|t| t.id).collect();
    let searcher = searcher(&state, &ctx).await;
    let counts = state
        .tag_counts
        .counts_for(ctx.owner, ctx.server_epoch, &searcher, &ids)
        .await;
    for tag in &mut tags {
        tag.asset_count = counts.get(&tag.id).copied();
    }
    Ok(Json(tags))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpsertTagsBody {
    tags: Vec<String>,
}

pub async fn upsert(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<UpsertTagsBody>,
) -> Result<Json<Vec<TagSummary>>, AppError> {
    let values: Vec<&str> = body.tags.iter().map(|value| value.trim()).collect();
    if values.is_empty() || values.iter().any(|value| value.is_empty()) {
        return Err(AppError::BadRequest("tag values required".into()));
    }
    let tags = ctx.immich.upsert_tags(&values).await?;
    state
        .tag_counts
        .clear_tenant(ctx.owner, ctx.server_epoch)
        .await;
    Ok(Json(tags))
}

pub async fn tag_asset(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((tag_id, asset_id)): Path<(Uuid, AssetKey)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = ctx
        .immich
        .set_asset_tag(tag_id, asset_id.source(), true)
        .await?;
    ensure_applied(&resp, "duplicate")?;
    state
        .tag_counts
        .invalidate(ctx.owner, ctx.server_epoch, tag_id)
        .await;
    state.search_caches.membership_changed();
    Ok(Json(resp))
}

pub async fn untag_asset(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((tag_id, asset_id)): Path<(Uuid, AssetKey)>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let resp = ctx
        .immich
        .set_asset_tag(tag_id, asset_id.source(), false)
        .await?;
    ensure_applied(&resp, "not_found")?;
    state
        .tag_counts
        .invalidate(ctx.owner, ctx.server_epoch, tag_id)
        .await;
    state.search_caches.membership_changed();
    Ok(Json(resp))
}

fn ensure_applied(resp: &[BulkIdResponse], idempotent: &str) -> Result<(), AppError> {
    let Some(refused) = resp
        .iter()
        .find(|r| !r.success && r.error.as_deref() != Some(idempotent))
    else {
        return Ok(());
    };
    let reason = refused
        .error
        .as_deref()
        .unwrap_or("unknown")
        .replace('_', " ");
    tracing::warn!(target: "app::error", asset = %refused.id, %reason, "immich refused the tag change");
    Err(AppError::UpstreamRejected(reason))
}
