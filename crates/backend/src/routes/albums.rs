use axum::Json;
use axum::extract::Path;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{AlbumDetail, AlbumSummary};
use crate::routes::auth::AuthCtx;

pub async fn list(ctx: AuthCtx) -> Result<Json<Vec<AlbumSummary>>, AppError> {
    let albums = ctx.immich.list_albums().await?;
    Ok(Json(albums))
}

pub async fn detail(ctx: AuthCtx, Path(id): Path<Uuid>) -> Result<Json<AlbumDetail>, AppError> {
    let album = ctx.immich.album(id).await?;
    Ok(Json(album))
}
