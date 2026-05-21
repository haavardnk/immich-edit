use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use raw_pipeline::edit_manifest::EditManifest;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::edits_store::{EditRecord, EditsStoreError, RENDERER_VERSION};
use crate::state::AppState;

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
        },
    };
    Ok(Json(record))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(manifest): Json<EditManifest>,
) -> Result<Json<EditRecord>, AppError> {
    let asset = state.immich.asset(id).await?;
    let saved = state
        .edits
        .put(id, manifest, asset.updated_at, asset.checksum)
        .await
        .map_err(map_err)?;
    Ok(Json(saved))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.edits.delete(id).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: EditsStoreError) -> AppError {
    tracing::error!(error = %err, "edits store");
    AppError::Internal
}
