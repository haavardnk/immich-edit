use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::CopyRecord;
use crate::state::AppState;

const MAX_COPY_NAME_LEN: usize = 64;

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CopySeed {
    #[default]
    Current,
    Neutral,
}

#[derive(Debug, Default, Deserialize)]
pub struct CreateCopyBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub from: CopySeed,
}

#[derive(Debug, Deserialize)]
pub struct RenameCopyBody {
    pub name: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<Vec<CopyRecord>>, AppError> {
    let copies = state.edits.list_copies(ctx.owner, id.source()).await?;
    Ok(Json(copies))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    body: Option<Json<CreateCopyBody>>,
) -> Result<(StatusCode, Json<CopyRecord>), AppError> {
    let body = body.map(|Json(b)| b).unwrap_or_default();
    let name = normalize_name(body.name)?;
    let asset = ctx.immich.asset(id.source()).await?;
    let copy = state
        .edits
        .create_copy(ctx.owner, id.source(), name.as_deref())
        .await?;
    if body.from == CopySeed::Current
        && let Some(source_edits) = state.edits.get(ctx.owner, id).await?
    {
        state
            .edits
            .put(
                ctx.owner,
                copy.id,
                source_edits.manifest,
                asset.updated_at,
                asset.checksum,
                Some("Create virtual copy"),
            )
            .await?;
    }
    Ok((StatusCode::CREATED, Json(copy)))
}

pub async fn rename(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<RenameCopyBody>,
) -> Result<Json<CopyRecord>, AppError> {
    let name = normalize_name(body.name)?;
    let copy = state
        .edits
        .rename_copy(ctx.owner, id, name.as_deref())
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(copy))
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<StatusCode, AppError> {
    if !state.edits.delete_copy(ctx.owner, id).await? {
        return Err(AppError::NotFound);
    }
    state
        .edited_thumb
        .purge_asset(ctx.server_epoch, ctx.owner, id)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

fn normalize_name(name: Option<String>) -> Result<Option<String>, AppError> {
    let Some(name) = name else {
        return Ok(None);
    };
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_COPY_NAME_LEN {
        return Err(AppError::BadRequest(format!(
            "name must be at most {MAX_COPY_NAME_LEN} characters"
        )));
    }
    Ok(Some(trimmed.to_string()))
}
