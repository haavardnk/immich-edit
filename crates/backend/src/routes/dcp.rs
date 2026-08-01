use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::error::AppError;
use crate::services::dcp_store::DcpMeta;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ImportParams {
    pub name: Option<String>,
    pub camera: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MatchParams {
    pub model: String,
    pub make: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<DcpMeta>>, AppError> {
    let profiles = state.dcp.list().await?;
    Ok(Json(profiles))
}

pub async fn match_camera(
    State(state): State<AppState>,
    Query(params): Query<MatchParams>,
) -> Result<Json<Option<DcpMeta>>, AppError> {
    if let Some(profile) = state.dcp.match_camera_meta(&params.model).await? {
        return Ok(Json(Some(profile)));
    }
    let make = params.make.as_deref().map(str::trim).unwrap_or_default();
    if make.is_empty() {
        return Ok(Json(None));
    }
    let qualified = format!("{make} {}", params.model);
    Ok(Json(state.dcp.match_camera_meta(&qualified).await?))
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
        .await?;
    Ok((StatusCode::CREATED, Json(meta)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.dcp.soft_delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
