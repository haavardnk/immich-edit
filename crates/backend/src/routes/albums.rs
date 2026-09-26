use axum::Json;
use axum::extract::Path;
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::{AlbumDetail, AlbumSummary, BulkIdResponse};
use crate::routes::auth::AuthCtx;

const MAX_ALBUM_ASSETS: usize = 10_000;

#[derive(Deserialize)]
pub struct AlbumAssetsBody {
    ids: Vec<AssetKey>,
}

pub async fn list(ctx: AuthCtx) -> Result<Json<Vec<AlbumSummary>>, AppError> {
    let albums = ctx.immich.list_albums().await?;
    Ok(Json(albums))
}

pub async fn detail(ctx: AuthCtx, Path(id): Path<Uuid>) -> Result<Json<AlbumDetail>, AppError> {
    let album = ctx.immich.album(id).await?;
    Ok(Json(album))
}

fn source_ids(body: AlbumAssetsBody) -> Result<Vec<Uuid>, AppError> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if body.ids.len() > MAX_ALBUM_ASSETS {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut ids: Vec<Uuid> = body.ids.iter().map(AssetKey::source).collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

pub async fn add_assets(
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<AlbumAssetsBody>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let ids = source_ids(body)?;
    Ok(Json(ctx.immich.add_assets_to_album(id, &ids).await?))
}

pub async fn remove_assets(
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<AlbumAssetsBody>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let ids = source_ids(body)?;
    Ok(Json(ctx.immich.remove_assets_from_album(id, &ids).await?))
}
