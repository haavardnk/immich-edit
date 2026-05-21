use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{AlbumDetail, AlbumSummary};
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<AlbumSummary>>, AppError> {
    let albums = state.immich.list_albums().await?;
    Ok(Json(albums))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AlbumDetail>, AppError> {
    let album = state.immich.album(id).await?;
    Ok(Json(album))
}
