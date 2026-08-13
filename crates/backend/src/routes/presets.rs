use axum::Json;
use axum::extract::{Path, State};
use raw_pipeline::edit_manifest::EditManifest;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::PresetRecord;
use crate::state::AppState;

const MAX_PRESET_NAME_LEN: usize = 80;
const MAX_GROUP_LEN: usize = 60;

#[derive(Debug, Deserialize)]
pub struct PresetBody {
    pub name: String,
    #[serde(default)]
    pub group_name: Option<String>,
    pub manifest: EditManifest,
}

struct ParsedPreset {
    name: String,
    group_name: Option<String>,
    manifest: EditManifest,
}

fn parse_body(body: PresetBody) -> Result<ParsedPreset, AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("preset name required".into()));
    }
    if name.len() > MAX_PRESET_NAME_LEN {
        return Err(AppError::BadRequest("preset name too long".into()));
    }
    let group_name = match body.group_name {
        Some(g) => {
            let trimmed = g.trim().to_string();
            if trimmed.is_empty() {
                None
            } else if trimmed.len() > MAX_GROUP_LEN {
                return Err(AppError::BadRequest("preset group too long".into()));
            } else {
                Some(trimmed)
            }
        }
        None => None,
    };
    Ok(ParsedPreset {
        name,
        group_name,
        manifest: body.manifest,
    })
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<PresetRecord>>, AppError> {
    let presets = state.edits.list_presets(ctx.owner).await?;
    Ok(Json(presets))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<PresetBody>,
) -> Result<Json<PresetRecord>, AppError> {
    let parsed = parse_body(body)?;
    let record = state
        .edits
        .create_preset(
            ctx.owner,
            &parsed.name,
            parsed.group_name.as_deref(),
            &parsed.manifest,
        )
        .await?;
    Ok(Json(record))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Json<PresetRecord>, AppError> {
    let record = state
        .edits
        .get_preset(ctx.owner, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(record))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<PresetBody>,
) -> Result<Json<PresetRecord>, AppError> {
    let parsed = parse_body(body)?;
    let record = state
        .edits
        .update_preset(
            ctx.owner,
            id,
            &parsed.name,
            parsed.group_name.as_deref(),
            &parsed.manifest,
        )
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(record))
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let deleted = state.edits.delete_preset(ctx.owner, id).await?;
    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
