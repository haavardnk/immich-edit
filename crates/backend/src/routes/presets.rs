use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::CancelTracker;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::frame::{JpegSubsampling, OutputFormat, RenderOptions};
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::headers::{self, attach_validators, etag_matches};
use crate::services::edit_merge::{MergeSections, merge_edits};
use crate::services::edits_store::PresetRecord;
use crate::services::render::RenderIdentity;
use crate::services::render_queue::{CancelOnDrop, RenderPriority};
use crate::state::AppState;

const MAX_PRESET_NAME_LEN: usize = 80;
const MAX_GROUP_LEN: usize = 60;
const THUMB_MAX_EDGE: u32 = 256;
const THUMB_MIN_EDGE: u32 = 64;
const THUMB_JPEG_QUALITY: u8 = 80;

#[derive(Debug, Deserialize)]
pub struct PresetThumbQuery {
    #[serde(default)]
    pub max: Option<u32>,
}

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

pub async fn thumb(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((asset_id, preset_id)): Path<(AssetKey, Uuid)>,
    Query(q): Query<PresetThumbQuery>,
    request_headers: HeaderMap,
) -> Result<Response, AppError> {
    let max_edge = q
        .max
        .unwrap_or(THUMB_MAX_EDGE)
        .clamp(THUMB_MIN_EDGE, THUMB_MAX_EDGE);
    let preset = state
        .edits
        .get_preset(ctx.owner, preset_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let current = state
        .edits
        .get_edits_or_default(ctx.owner, asset_id)
        .await?;
    let look = preset.manifest.to_edits();
    let render_revision = state.render.render_revision().await?;
    let etag = headers::etag(&format!(
        "{}-{}-{}-{}-{}",
        current.stable_hash(),
        look.stable_hash(),
        max_edge,
        ctx.server_epoch,
        render_revision
    ))?;
    if etag_matches(&request_headers, &etag) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        attach_validators(&mut resp, etag);
        return Ok(resp);
    }
    let edits = merge_edits(current, look, MergeSections::look_only()).clamped();
    let opts = RenderOptions {
        max_edge,
        quality: false,
        output: OutputFormat::Jpeg {
            quality: THUMB_JPEG_QUALITY,
            subsampling: JpegSubsampling::Chroma420,
        },
        ..Default::default()
    };
    let token = CancelTracker::new().next();
    let guard = CancelOnDrop::new(token.clone());
    let work = state.render.render(
        RenderIdentity::from(&ctx),
        ctx.immich.clone(),
        asset_id.source(),
        edits,
        opts,
        Some(token),
    );
    let rendered = state
        .queue
        .run(RenderPriority::Background, work)
        .await
        .ok_or(AppError::Superseded)??;
    guard.disarm();
    let mut resp = Response::new(Body::from(rendered.bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    attach_validators(&mut resp, etag);
    Ok(resp)
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
