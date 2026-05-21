use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FolderQuery {
    pub path: Option<String>,
}

pub async fn paths(State(state): State<AppState>) -> Result<Json<Vec<String>>, AppError> {
    let paths = state.immich.folder_paths().await?;
    Ok(Json(paths))
}

pub async fn assets(
    State(state): State<AppState>,
    Query(q): Query<FolderQuery>,
) -> Result<Json<Vec<AssetDetail>>, AppError> {
    let path = q.path.unwrap_or_default();
    let assets = state.immich.folder_assets(&path).await?;
    Ok(Json(assets))
}
