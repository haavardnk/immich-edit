use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::ThumbSize;
use crate::immich::dto::AssetDetail;
use crate::state::AppState;

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetDetail>, AppError> {
    let asset = state.immich.asset(id).await?;
    Ok(Json(asset))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<AssetDetail>, AppError> {
    let asset = state.immich.update_asset(id, &body).await?;
    Ok(Json(asset))
}

#[derive(Debug, Deserialize)]
pub struct ThumbQuery {
    #[serde(default = "default_thumb_size")]
    pub size: String,
}

fn default_thumb_size() -> String {
    "preview".into()
}

pub async fn thumbnail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ThumbQuery>,
) -> Result<Response, AppError> {
    let size = ThumbSize::parse(&q.size)
        .ok_or_else(|| AppError::BadRequest(format!("invalid size: {}", q.size)))?;
    let (bytes, content_type) = state.immich.thumbnail(id, size).await?;
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("image/jpeg")),
    );
    Ok(resp.into_response())
}
