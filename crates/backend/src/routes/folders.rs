use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::routes::auth::AuthCtx;
use crate::services::copy_expand::expand_assets;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FolderQuery {
    pub path: Option<String>,
}

pub async fn paths(ctx: AuthCtx) -> Result<Json<Vec<String>>, AppError> {
    let paths = ctx.immich.folder_paths().await?;
    Ok(Json(paths))
}

pub async fn assets(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Query(q): Query<FolderQuery>,
) -> Result<Json<Vec<AssetDetail>>, AppError> {
    let path = q.path.unwrap_or_default();
    let assets = ctx.immich.folder_assets(&path).await?;
    let assets = expand_assets(&state.edits, ctx.owner, assets).await?;
    Ok(Json(assets))
}
