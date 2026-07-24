use axum::Json;
use axum::extract::Query;
use serde::Deserialize;

use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::routes::auth::AuthCtx;

#[derive(Debug, Deserialize)]
pub struct FolderQuery {
    pub path: Option<String>,
}

pub async fn paths(ctx: AuthCtx) -> Result<Json<Vec<String>>, AppError> {
    let paths = ctx.immich.folder_paths().await?;
    Ok(Json(paths))
}

pub async fn assets(
    ctx: AuthCtx,
    Query(q): Query<FolderQuery>,
) -> Result<Json<Vec<AssetDetail>>, AppError> {
    let path = q.path.unwrap_or_default();
    let assets = ctx.immich.folder_assets(&path).await?;
    Ok(Json(assets))
}
