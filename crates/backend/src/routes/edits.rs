use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::{EditHistoryEntry, EditRecord, EditedAssetEntry};
use crate::services::render::RenderIdentity;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PutEditsBody {
    Wrapped {
        manifest: EditManifest,
        #[serde(default)]
        action: Option<String>,
    },
    Raw(EditManifest),
}

impl PutEditsBody {
    fn split(self) -> (EditManifest, Option<String>) {
        match self {
            PutEditsBody::Wrapped { manifest, action } => (manifest, action),
            PutEditsBody::Raw(manifest) => (manifest, None),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ActionBody {
    #[serde(default)]
    pub action: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<EditedAssetEntry>>, AppError> {
    let entries = state.edits.list_edited_assets(ctx.owner).await?;
    Ok(Json(entries))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<EditRecord>, AppError> {
    let record = state.edits.get(ctx.owner, id).await?;
    let record = record.unwrap_or_else(|| EditRecord::empty(id));
    Ok(Json(record))
}

pub async fn put(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    headers: HeaderMap,
    Json(body): Json<PutEditsBody>,
) -> Result<Response, AppError> {
    let (manifest, action) = body.split();
    let if_match = headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());
    if let Some(expected) = if_match.as_deref()
        && let Some(current) = state
            .edits
            .if_match_conflict(ctx.owner, id, expected)
            .await?
    {
        return Ok((StatusCode::CONFLICT, Json(current)).into_response());
    }
    let asset = ctx.immich.asset(id.source()).await?;
    let saved = state
        .edits
        .put(
            ctx.owner,
            id,
            manifest,
            asset.updated_at,
            asset.checksum,
            action.as_deref(),
        )
        .await?;
    Ok(Json(saved).into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    headers: HeaderMap,
    body: Option<Json<ActionBody>>,
) -> Result<Response, AppError> {
    let if_match = headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());
    if let Some(expected) = if_match.as_deref()
        && let Some(current) = state
            .edits
            .if_match_conflict(ctx.owner, id, expected)
            .await?
    {
        return Ok((StatusCode::CONFLICT, Json(current)).into_response());
    }
    let action = body
        .and_then(|Json(b)| b.action)
        .unwrap_or_else(|| "Reset".to_string());
    state
        .edits
        .delete(ctx.owner, id, Some(action.as_str()))
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn auto(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    body: axum::body::Bytes,
) -> Result<Json<Edits>, AppError> {
    let context = if body.is_empty() {
        Edits::default()
    } else {
        serde_json::from_slice::<Edits>(&body)
            .map_err(|e| AppError::BadRequest(format!("invalid edits body: {e}")))?
    };
    let frame = state
        .render
        .quality_frame(RenderIdentity::from(&ctx), &ctx.immich, id.source())
        .await
        .map_err(AppError::from)?;
    let edits =
        tokio::task::spawn_blocking(move || raw_pipeline::auto::auto_adjust(&frame, &context))
            .await
            .map_err(|_| AppError::Internal)?;
    Ok(Json(edits))
}

pub async fn history(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<Vec<EditHistoryEntry>>, AppError> {
    let entries = state.edits.list_history(ctx.owner, id).await?;
    Ok(Json(entries))
}

#[derive(Debug, Deserialize)]
pub struct RestoreBody {
    pub entry_id: i64,
}

pub async fn restore(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<RestoreBody>,
) -> Result<Response, AppError> {
    let Some(entry) = state
        .edits
        .get_history_entry(ctx.owner, id, body.entry_id)
        .await?
    else {
        return Err(AppError::NotFound);
    };
    let asset = ctx.immich.asset(id.source()).await?;
    let saved = state
        .edits
        .restore_to_entry(ctx.owner, id, &entry, asset.updated_at, asset.checksum)
        .await?;
    match saved {
        Some(record) => Ok(Json(record).into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}
