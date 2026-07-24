use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::error::AppError;
use crate::services::dcp_store::{DcpMeta, DcpStoreError};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportParams {
    pub name: Option<String>,
    pub camera: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<DcpMeta>>, AppError> {
    let profiles = state.dcp.list().await.map_err(map_err)?;
    Ok(Json(profiles))
}

pub async fn import(
    State(state): State<AppState>,
    Query(params): Query<ImportParams>,
    body: Bytes,
) -> Result<(StatusCode, Json<DcpMeta>), AppError> {
    let meta = state
        .dcp
        .import(
            params.name.as_deref(),
            params.camera.as_deref(),
            false,
            &body,
        )
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(meta)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.dcp.soft_delete(&id).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: DcpStoreError) -> AppError {
    match err {
        DcpStoreError::NotFound => AppError::NotFound,
        DcpStoreError::Invalid(m) => AppError::BadRequest(m),
        DcpStoreError::Duplicate(meta) => {
            AppError::Conflict(format!("dcp already exists: {}", meta.id))
        }
        e => {
            tracing::error!(error = %e, "dcp store");
            AppError::Internal
        }
    }
}
