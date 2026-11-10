use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use raw_pipeline::edit_manifest::EditManifest;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::names::display_name;
use crate::routes::auth::AuthCtx;
use crate::routes::edits::write_manifest;
use crate::routes::headers;
use crate::services::edits_store::EditSnapshot;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSnapshotBody {
    pub name: String,
    pub manifest: EditManifest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSnapshotBody {
    pub name: Option<String>,
    pub manifest: Option<EditManifest>,
}

#[derive(Debug, Serialize)]
pub struct SnapshotItem {
    pub id: Uuid,
    pub name: String,
    pub manifest_hash: String,
    pub manifest: EditManifest,
    pub created_at: String,
    pub updated_at: String,
}

impl From<EditSnapshot> for SnapshotItem {
    fn from(snapshot: EditSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            manifest_hash: snapshot.manifest_hash,
            manifest: EditManifest::from_edits(&snapshot.edits),
            created_at: snapshot.created_at,
            updated_at: snapshot.updated_at,
        }
    }
}

fn snapshot_name(raw: &str) -> Result<String, AppError> {
    display_name(raw, "snapshot name").map_err(AppError::BadRequest)
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<Vec<SnapshotItem>>, AppError> {
    let snapshots = state.edits.list_snapshots(ctx.owner, id).await?;
    Ok(Json(
        snapshots.into_iter().map(SnapshotItem::from).collect(),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<CreateSnapshotBody>,
) -> Result<Json<SnapshotItem>, AppError> {
    let name = snapshot_name(&body.name)?;
    let snapshot = state
        .edits
        .create_snapshot(ctx.owner, id, &name, &body.manifest)
        .await?;
    Ok(Json(snapshot.into()))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((id, snapshot_id)): Path<(AssetKey, Uuid)>,
    Json(body): Json<UpdateSnapshotBody>,
) -> Result<Json<SnapshotItem>, AppError> {
    if body.name.is_none() && body.manifest.is_none() {
        return Err(AppError::BadRequest(
            "snapshot update needs a name or a manifest".into(),
        ));
    }
    let name = body.name.as_deref().map(snapshot_name).transpose()?;
    state
        .edits
        .update_snapshot(
            ctx.owner,
            id,
            snapshot_id,
            name.as_deref(),
            body.manifest.as_ref(),
        )
        .await?
        .map(|snapshot| Json(snapshot.into()))
        .ok_or(AppError::NotFound)
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((id, snapshot_id)): Path<(AssetKey, Uuid)>,
) -> Result<StatusCode, AppError> {
    if state
        .edits
        .delete_snapshot(ctx.owner, id, snapshot_id)
        .await?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn restore(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((id, snapshot_id)): Path<(AssetKey, Uuid)>,
    request_headers: HeaderMap,
) -> Result<Response, AppError> {
    let snapshot = state
        .edits
        .get_snapshot(ctx.owner, id, snapshot_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let action = format!("Restore snapshot: {}", snapshot.name);
    let if_match = headers::if_match(&request_headers);
    write_manifest(
        &state,
        &ctx,
        id,
        EditManifest::from_edits(&snapshot.edits),
        Some(action.as_str()),
        if_match.as_deref(),
    )
    .await
}
