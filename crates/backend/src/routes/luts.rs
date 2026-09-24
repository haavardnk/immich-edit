use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;

use crate::error::AppError;
use crate::routes::auth::AdminCtx;
use crate::routes::immutable_bytes;
use crate::services::lut_store::LutMeta;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportParams {
    pub name: String,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<LutMeta>>, AppError> {
    let luts = state.luts.list().await?;
    Ok(Json(luts))
}

pub async fn cube(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let bytes = state.luts.source_bytes(&id).await?;
    Ok(immutable_bytes(bytes))
}

pub async fn import(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Query(params): Query<ImportParams>,
    body: Bytes,
) -> Result<(StatusCode, Json<LutMeta>), AppError> {
    let meta = state.luts.import(&params.name, &body).await?;
    Ok((StatusCode::CREATED, Json(meta)))
}

pub async fn delete(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.luts.soft_delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
