use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use raw_pipeline::edits::{Edits, Sidecar};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::edits_store::EditsStoreError;
use crate::state::AppState;

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Sidecar>, AppError> {
    let sidecar = state.edits.get(id).await.map_err(map_err)?;
    let sidecar = match sidecar {
        Some(s) => s,
        None => Sidecar {
            schema_version: 1,
            asset_id: id,
            immich_updated_at: None,
            immich_checksum: None,
            renderer_version: crate::services::edits_store::RENDERER_VERSION.into(),
            edits: Edits::default(),
            updated_at: String::new(),
        },
    };
    Ok(Json(sidecar))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(edits): Json<Edits>,
) -> Result<Json<Sidecar>, AppError> {
    let asset = state.immich.asset(id).await?;
    let saved = state
        .edits
        .put(id, edits, asset.updated_at, asset.checksum)
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
