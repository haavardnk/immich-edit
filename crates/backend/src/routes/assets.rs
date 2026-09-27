use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::client::ThumbSize;
use crate::immich::dto::AssetDetail;
use crate::routes::auth::AuthCtx;
use crate::routes::headers;

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
    #[serde(default)]
    pub size: ThumbSize,
}

pub async fn thumbnail(
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(q): Query<ThumbQuery>,
) -> Result<Response, AppError> {
    let (bytes, content_type) = ctx.immich.thumbnail(id.source(), q.size).await?;
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, headers::header_value(&content_type)?);
    Ok(resp.into_response())
}
