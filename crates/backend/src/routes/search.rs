use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::{AssetSummary, SearchAssets, SearchStatistics};
use crate::routes::auth::AuthCtx;
use crate::services::copy_expand::expand_assets;
use crate::services::search_window::{self, MAX_RADIUS};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowBody {
    asset_id: AssetKey,
    radius: u32,
    #[serde(default)]
    query: Map<String, Value>,
}

#[derive(Serialize)]
pub struct SearchWindow {
    items: Vec<AssetSummary>,
}

pub async fn metadata(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let mut assets = ctx.immich.search_metadata(&body).await?;
    assets.items = expand_assets(&state.edits, ctx.owner, assets.items).await?;
    Ok(Json(assets))
}

pub async fn smart(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchAssets>, AppError> {
    let mut assets = ctx.immich.search_smart(&body).await?;
    assets.items = expand_assets(&state.edits, ctx.owner, assets.items).await?;
    Ok(Json(assets))
}

pub async fn statistics(
    ctx: AuthCtx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SearchStatistics>, AppError> {
    let stats = ctx.immich.search_statistics(&body).await?;
    Ok(Json(stats))
}

pub async fn window(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<WindowBody>,
) -> Result<Json<SearchWindow>, AppError> {
    if !(1..=MAX_RADIUS).contains(&body.radius) {
        return Err(AppError::BadRequest(format!(
            "radius must be 1 to {MAX_RADIUS}"
        )));
    }
    let anchor = body.asset_id.source();
    let items = search_window::around(&ctx.immich, body.query, anchor, body.radius).await?;
    let items = expand_assets(&state.edits, ctx.owner, items).await?;
    Ok(Json(SearchWindow { items }))
}
