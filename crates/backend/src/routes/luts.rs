use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::error::AppError;
use crate::services::lut_store::{LutMeta, LutStoreError};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportParams {
    pub name: String,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<LutMeta>>, AppError> {
    let luts = state.luts.list().await.map_err(map_err)?;
    Ok(Json(luts))
}

pub async fn import(
    State(state): State<AppState>,
    Query(params): Query<ImportParams>,
    body: Bytes,
) -> Result<(StatusCode, Json<LutMeta>), AppError> {
    let meta = state
        .luts
        .import(&params.name, &body)
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(meta)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.luts.soft_delete(&id).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: LutStoreError) -> AppError {
    match err {
        LutStoreError::NotFound => AppError::NotFound,
        LutStoreError::Invalid(m) => AppError::BadRequest(m),
        LutStoreError::Duplicate(meta) => {
            AppError::Conflict(format!("lut already exists: {}", meta.id))
        }
        e => {
            tracing::error!(error = %e, "lut store");
            AppError::Internal
        }
    }
}
