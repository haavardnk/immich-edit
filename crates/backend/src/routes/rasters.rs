use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::AppError;
use crate::services::raster_store::{RasterMeta, RasterStoreError};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UploadParams {
    pub width: u32,
    pub height: u32,
}

pub async fn upload(
    State(state): State<AppState>,
    Query(params): Query<UploadParams>,
    body: Bytes,
) -> Result<Json<RasterMeta>, AppError> {
    let meta = state
        .rasters
        .store(&body, params.width, params.height)
        .await
        .map_err(map_err)?;
    Ok(Json(meta))
}

pub async fn get(
    State(state): State<AppState>,
    Path(raster_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (meta, bytes) = state.rasters.load(&raster_id).await.map_err(map_err)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert("x-raster-width", HeaderValue::from(meta.width));
    headers.insert("x-raster-height", HeaderValue::from(meta.height));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    Ok((StatusCode::OK, headers, bytes))
}

pub async fn meta(
    State(state): State<AppState>,
    Path(raster_id): Path<String>,
) -> Result<Json<RasterMeta>, AppError> {
    let m = state.rasters.meta(&raster_id).await.map_err(map_err)?;
    Ok(Json(m))
}

fn map_err(err: RasterStoreError) -> AppError {
    match err {
        RasterStoreError::NotFound => AppError::NotFound,
        RasterStoreError::Invalid(m) => AppError::BadRequest(m),
        e => {
            tracing::error!(error = %e, "raster store");
            AppError::Internal
        }
    }
}
