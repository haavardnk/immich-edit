use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::error::AppError;
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

pub async fn import(
    State(state): State<AppState>,
    Query(params): Query<ImportParams>,
    body: Bytes,
) -> Result<(StatusCode, Json<LutMeta>), AppError> {
    let meta = state.luts.import(&params.name, &body).await?;
    Ok((StatusCode::CREATED, Json(meta)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.luts.soft_delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
