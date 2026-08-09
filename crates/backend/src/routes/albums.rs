use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{AlbumDetail, AlbumSummary};
use crate::routes::auth::AuthCtx;
use crate::services::copy_expand::expand_assets;
use crate::state::AppState;

pub async fn list(ctx: AuthCtx) -> Result<Json<Vec<AlbumSummary>>, AppError> {
    let albums = ctx.immich.list_albums().await?;
    Ok(Json(albums))
}

pub async fn detail(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Json<AlbumDetail>, AppError> {
    let mut album = ctx.immich.album(id).await?;
    album.assets = expand_assets(&state.edits, ctx.owner, album.assets).await?;
    Ok(Json(album))
}
