use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::{AssetSummary, SearchStatistics};
use crate::routes::auth::AuthCtx;
use crate::services::copy_expand::expand_assets;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{CountQuery, PageQuery, SmartQuery, Sort};
use crate::services::search::window::{self, MAX_RADIUS};
use crate::services::search::{SearchPage, Searcher};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowBody {
    asset_id: AssetKey,
    radius: u32,
    #[serde(default)]
    filter: BrowseFilter,
    #[serde(default)]
    sort: Sort,
}

#[derive(Serialize)]
pub struct SearchWindow {
    items: Vec<AssetSummary>,
}

async fn searcher(state: &AppState, ctx: &AuthCtx) -> Searcher {
    Searcher::connect(&state.capabilities, ctx.server_epoch, &ctx.immich).await
}

pub async fn metadata(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(query): Json<PageQuery>,
) -> Result<Json<SearchPage>, AppError> {
    let mut page = searcher(&state, &ctx).await.page(&query).await?;
    page.items = expand_assets(&state.edits, ctx.owner, page.items).await?;
    Ok(Json(page))
}

pub async fn smart(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(query): Json<SmartQuery>,
) -> Result<Json<SearchPage>, AppError> {
    let mut page = searcher(&state, &ctx).await.smart(&query).await?;
    page.items = expand_assets(&state.edits, ctx.owner, page.items).await?;
    Ok(Json(page))
}

pub async fn statistics(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(query): Json<CountQuery>,
) -> Result<Json<SearchStatistics>, AppError> {
    let total = searcher(&state, &ctx).await.count(&query.filter).await?;
    Ok(Json(SearchStatistics { total }))
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
    let searcher = searcher(&state, &ctx).await;
    let anchor = body.asset_id.source();
    let items = window::around(&searcher, &body.filter, body.sort, anchor, body.radius).await?;
    let items = expand_assets(&state.edits, ctx.owner, items).await?;
    Ok(Json(SearchWindow { items }))
}
