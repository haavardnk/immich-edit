use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::edits_store::{
    EditHistoryEntry, EditRecord, EditedAssetEntry, EditsStoreError, RENDERER_VERSION,
};
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

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<EditedAssetEntry>>, AppError> {
    let entries = state.edits.list_edited_assets().await.map_err(map_err)?;
    Ok(Json(entries))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EditRecord>, AppError> {
    let record = state.edits.get(id).await.map_err(map_err)?;
    let record = match record {
        Some(r) => r,
        None => EditRecord {
            schema_version: 2,
            asset_id: id,
            immich_updated_at: None,
            immich_checksum: None,
            renderer_version: RENDERER_VERSION.into(),
            manifest: EditManifest::default(),
            updated_at: String::new(),
            hash: Edits::default().stable_hash(),
        },
    };
    Ok(Json(record))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<PutEditsBody>,
) -> Result<Response, AppError> {
    let (manifest, action) = body.split();
    let if_match = headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());
    if let Some(expected) = if_match.as_deref() {
        let current = state.edits.get(id).await?;
        let current_hash = match &current {
            Some(r) => r.hash.as_str(),
            None => "",
        };
        let default_hash = Edits::default().stable_hash();
        let actual = if current_hash.is_empty() {
            default_hash.as_str()
        } else {
            current_hash
        };
        if expected != actual {
            let body = current.unwrap_or_else(|| EditRecord {
                schema_version: 2,
                asset_id: id,
                immich_updated_at: None,
                immich_checksum: None,
                renderer_version: RENDERER_VERSION.into(),
                manifest: EditManifest::default(),
                updated_at: String::new(),
                hash: default_hash,
            });
            return Ok((StatusCode::CONFLICT, Json(body)).into_response());
        }
    }
    let asset = state.immich.asset(id).await?;
    let saved = state
        .edits
        .put(
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
    Path(id): Path<Uuid>,
    body: Option<Json<ActionBody>>,
) -> Result<StatusCode, AppError> {
    let action = body
        .and_then(|Json(b)| b.action)
        .unwrap_or_else(|| "Reset".to_string());
    state
        .edits
        .delete(id, Some(action.as_str()))
        .await
        .map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn auto(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    body: axum::body::Bytes,
) -> Result<Json<Edits>, AppError> {
    let context = if body.is_empty() {
        Edits::default()
    } else {
        serde_json::from_slice::<Edits>(&body).unwrap_or_default()
    };
    let frame = state.render.quality_frame(id).await.map_err(|e| match e {
        crate::services::render::RenderError::Upstream(u) => u.into(),
        crate::services::render::RenderError::Pipeline(p) => {
            tracing::error!(error = %p, "auto-adjust decode");
            AppError::Internal
        }
    })?;
    let edits =
        tokio::task::spawn_blocking(move || raw_pipeline::auto::auto_adjust(&frame, &context))
            .await
            .map_err(|_| AppError::Internal)?;
    Ok(Json(edits))
}

fn map_err(err: EditsStoreError) -> AppError {
    tracing::error!(error = %err, "edits store");
    AppError::Internal
}

pub async fn history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<EditHistoryEntry>>, AppError> {
    let entries = state.edits.list_history(id).await?;
    Ok(Json(entries))
}

#[derive(Debug, Deserialize)]
pub struct RestoreBody {
    pub entry_id: i64,
}

pub async fn restore(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<RestoreBody>,
) -> Result<Response, AppError> {
    let Some(entry) = state.edits.get_history_entry(id, body.entry_id).await? else {
        return Err(AppError::NotFound);
    };
    let saved = state.edits.restore_to_entry(id, &entry).await?;
    match saved {
        Some(record) => Ok(Json(record).into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}
