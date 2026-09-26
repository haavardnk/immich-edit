use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::ExportPresetRecord;
use crate::state::AppState;

const MAX_NAME_LEN: usize = 80;
const MAX_FORM_BYTES: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
pub struct ExportPresetBody {
    pub name: String,
    pub form: serde_json::Value,
}

fn parse_body(body: ExportPresetBody) -> Result<(String, serde_json::Value), AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("export preset name required".into()));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(AppError::BadRequest("export preset name too long".into()));
    }
    if !body.form.is_object() {
        return Err(AppError::BadRequest(
            "export preset form must be an object".into(),
        ));
    }
    if body.form.to_string().len() > MAX_FORM_BYTES {
        return Err(AppError::BadRequest("export preset form too large".into()));
    }
    Ok((name, body.form))
}

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<ExportPresetRecord>>, AppError> {
    Ok(Json(state.edits.list_export_presets(ctx.owner).await?))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<ExportPresetBody>,
) -> Result<Json<ExportPresetRecord>, AppError> {
    let (name, form) = parse_body(body)?;
    Ok(Json(
        state
            .edits
            .create_export_preset(ctx.owner, &name, &form)
            .await?,
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<ExportPresetBody>,
) -> Result<Json<ExportPresetRecord>, AppError> {
    let (name, form) = parse_body(body)?;
    state
        .edits
        .update_export_preset(ctx.owner, id, &name, &form)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.edits.delete_export_preset(ctx.owner, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
