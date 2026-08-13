use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::client::ThumbSize;
use crate::immich::dto::AssetDetail;
use crate::routes::auth::AuthCtx;

pub async fn detail(ctx: AuthCtx, Path(id): Path<AssetKey>) -> Result<Json<AssetDetail>, AppError> {
    let asset = ctx.immich.asset(id.source()).await?;
    Ok(Json(patch_copy(asset, id)))
}

pub async fn update(
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<AssetDetail>, AppError> {
    let asset = ctx.immich.update_asset(id.source(), &body).await?;
    Ok(Json(patch_copy(asset, id)))
}

fn patch_copy(mut asset: AssetDetail, id: AssetKey) -> AssetDetail {
    if id.is_copy() {
        asset.id = id;
        asset.copy_of = Some(id.source());
    }
    asset
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
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(q): Query<ThumbQuery>,
) -> Result<Response, AppError> {
    let size = ThumbSize::parse(&q.size)
        .ok_or_else(|| AppError::BadRequest(format!("invalid size: {}", q.size)))?;
    let (bytes, content_type) = ctx.immich.thumbnail(id.source(), size).await?;
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("image/jpeg")),
    );
    Ok(resp.into_response())
}
