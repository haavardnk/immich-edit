use axum::Json;
use axum::extract::{Path, State};
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use segment::{BakeParams, ModelKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::preview::map_render_err;
use crate::services::render::RenderIdentity;
use crate::services::segment::SegmentServiceError;
use crate::state::AppState;

const MAX_REFINE_PX: f32 = 128.0;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub kind: String,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub raster_id: String,
    pub prob_raster_id: String,
    pub width: u32,
    pub height: u32,
    pub model_id: String,
    pub backend: &'static str,
    pub elapsed_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct RebakeRequest {
    pub asset_id: Uuid,
    pub prob_raster_id: String,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
}

#[derive(Debug, Serialize)]
pub struct RebakeResponse {
    pub raster_id: String,
    pub width: u32,
    pub height: u32,
}

fn parse_kind(kind: &str) -> Result<ModelKind, AppError> {
    match kind {
        "subject" => Ok(ModelKind::Subject),
        "people" => Ok(ModelKind::People),
        "sky" => Ok(ModelKind::Sky),
        "depth" => Ok(ModelKind::Depth),
        other => Err(AppError::BadRequest(format!("unknown mask kind: {other}"))),
    }
}

fn bake_params(grow: f32, feather: f32) -> Result<BakeParams, AppError> {
    if !grow.is_finite() || !feather.is_finite() {
        return Err(AppError::BadRequest(
            "grow and feather must be finite".into(),
        ));
    }
    if !(-MAX_REFINE_PX..=MAX_REFINE_PX).contains(&grow) {
        return Err(AppError::BadRequest(format!("grow out of range: {grow}")));
    }
    if !(0.0..=MAX_REFINE_PX).contains(&feather) {
        return Err(AppError::BadRequest(format!(
            "feather out of range: {feather}"
        )));
    }
    Ok(BakeParams {
        grow,
        feather,
        ..Default::default()
    })
}

fn map_segment_err(err: SegmentServiceError) -> AppError {
    match err {
        SegmentServiceError::Disabled => {
            AppError::BadRequest("segmentation is disabled on this server".into())
        }
        SegmentServiceError::ModelMissing(kind) => {
            AppError::BadRequest(format!("no model installed for {kind}"))
        }
        _ => {
            tracing::error!(error = %err, "segmentation");
            AppError::Internal
        }
    }
}

async fn scene_render(
    state: &AppState,
    ctx: &AuthCtx,
    asset_id: Uuid,
) -> Result<(Vec<u8>, u32, u32), AppError> {
    let mut edits = state
        .edits
        .get_edits_or_default(ctx.owner, asset_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "edits store");
            AppError::Internal
        })?;
    edits.geometry = Default::default();
    edits.lens = Default::default();
    edits.effects = Default::default();
    edits.masks.clear();

    let identity = RenderIdentity {
        owner: ctx.owner,
        server_epoch: ctx.server_epoch,
    };
    let opts = RenderOptions {
        max_edge: state.segment.max_edge(),
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let rendered = state
        .render
        .render(identity, ctx.immich.clone(), asset_id, edits, opts, None)
        .await
        .map_err(map_render_err)?;
    Ok((rendered.bytes, rendered.width, rendered.height))
}

pub async fn generate(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(asset_id): Path<Uuid>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    if !state.segment.enabled() {
        return Err(AppError::BadRequest(
            "segmentation is disabled on this server".into(),
        ));
    }
    let kind = parse_kind(&req.kind)?;
    let params = bake_params(req.grow, req.feather)?;

    let (rgb8, width, height) = scene_render(&state, &ctx, asset_id).await?;
    let result = state
        .segment
        .generate(kind, rgb8, width, height, params)
        .await
        .map_err(map_segment_err)?;

    let baked = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &result.bytes, width, height)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "raster store");
            AppError::Internal
        })?;
    let prob = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &result.prob, width, height)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "raster store");
            AppError::Internal
        })?;

    Ok(Json(GenerateResponse {
        raster_id: baked.raster_id,
        prob_raster_id: prob.raster_id,
        width,
        height,
        model_id: result.model_id,
        backend: result.backend,
        elapsed_ms: result.elapsed_ms,
    }))
}

pub async fn rebake(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(req): Json<RebakeRequest>,
) -> Result<Json<RebakeResponse>, AppError> {
    let params = bake_params(req.grow, req.feather)?;
    let (meta, prob) = state
        .rasters
        .load(ctx.server_epoch, ctx.owner, &req.prob_raster_id)
        .await
        .map_err(|_| AppError::NotFound)?;

    let (rgb8, width, height) = scene_render(&state, &ctx, req.asset_id).await?;
    if meta.width != width || meta.height != height {
        return Err(AppError::Conflict(
            "probability raster does not match the current scene size".into(),
        ));
    }

    let bytes = state
        .segment
        .rebake(prob, rgb8, width, height, params)
        .await
        .map_err(map_segment_err)?;
    let baked = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &bytes, width, height)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "raster store");
            AppError::Internal
        })?;

    Ok(Json(RebakeResponse {
        raster_id: baked.raster_id,
        width,
        height,
    }))
}
