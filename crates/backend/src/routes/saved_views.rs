use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::SavedViewRecord;
use crate::state::AppState;

mod validate;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedViewBody {
    pub name: String,
    pub url: String,
}

fn parse_body(body: SavedViewBody) -> Result<(String, String), AppError> {
    let name = validate::name(&body.name).map_err(AppError::BadRequest)?;
    validate::url(&body.url).map_err(AppError::BadRequest)?;
    Ok((name, body.url))
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<SavedViewRecord>>, AppError> {
    Ok(Json(state.edits.list_saved_views(ctx.owner).await?))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<SavedViewBody>,
) -> Result<Json<SavedViewRecord>, AppError> {
    let (name, url) = parse_body(body)?;
    Ok(Json(
        state
            .edits
            .create_saved_view(ctx.owner, &name, &url)
            .await?,
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<SavedViewBody>,
) -> Result<Json<SavedViewRecord>, AppError> {
    let (name, url) = parse_body(body)?;
    state
        .edits
        .update_saved_view(ctx.owner, id, &name, &url)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.edits.delete_saved_view(ctx.owner, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
