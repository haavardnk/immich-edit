use axum::Json;
use axum::extract::{Path, State};
use ml::{BakeParams, BoxPrompt, ClickPoint, ModelKind, RangeWindow};
use raw_pipeline::edits::{MAX_REFINE_PX, N_MAX_CLICK_POINTS};
use serde::{Deserialize, Serialize};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::embedding_cache::EmbeddingKey;
use crate::services::mask_scene::{SceneImage, combine_coverage, render_scene};
use crate::services::render::RenderIdentity;
use crate::services::segment::SegmentServiceError;
use crate::state::AppState;

const MIN_BOX: f32 = 0.005;

#[derive(Debug, Deserialize)]
pub struct ClickPointBody {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_true")]
    pub positive: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ClickRequest {
    #[serde(default)]
    pub points: Vec<ClickPointBody>,
    #[serde(default)]
    pub bbox: Option<BoxBody>,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
    #[serde(default)]
    pub base_raster_id: Option<String>,
    #[serde(default)]
    pub subtract: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BoxBody {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

fn scene_box(b: &BoxBody) -> Result<BoxBody, AppError> {
    let unit = [b.x0, b.y0, b.x1, b.y1]
        .iter()
        .all(|v| (0.0..=1.0).contains(v));
    if !unit {
        return Err(AppError::BadRequest(
            "box coordinates must be within 0..1".into(),
        ));
    }
    let out = BoxBody {
        x0: b.x0.min(b.x1),
        y0: b.y0.min(b.y1),
        x1: b.x0.max(b.x1),
        y1: b.y0.max(b.y1),
    };
    if out.x1 - out.x0 < MIN_BOX || out.y1 - out.y0 < MIN_BOX {
        return Err(AppError::BadRequest("box is too small".into()));
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
pub struct RangeBody {
    pub min: f32,
    pub max: f32,
    #[serde(default)]
    pub softness: f32,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub kind: String,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
    #[serde(default)]
    pub range: Option<RangeBody>,
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
    pub asset_id: AssetKey,
    pub prob_raster_id: String,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
    #[serde(default)]
    pub range: Option<RangeBody>,
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
        "semantic" => Ok(ModelKind::Semantic),
        other => Err(AppError::BadRequest(format!("unknown mask kind: {other}"))),
    }
}

fn bake_params(grow: f32, feather: f32, range: Option<&RangeBody>) -> Result<BakeParams, AppError> {
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
    let window = match range {
        Some(r) => {
            if !r.min.is_finite() || !r.max.is_finite() || !r.softness.is_finite() {
                return Err(AppError::BadRequest("range must be finite".into()));
            }
            if !(0.0..=1.0).contains(&r.min) || !(0.0..=1.0).contains(&r.max) {
                return Err(AppError::BadRequest("range must be within 0..1".into()));
            }
            if !(0.0..=1.0).contains(&r.softness) {
                return Err(AppError::BadRequest("softness must be within 0..1".into()));
            }
            let window = RangeWindow {
                min: r.min,
                max: r.max,
                softness: r.softness,
            };
            if window.is_full() { None } else { Some(window) }
        }
        None => None,
    };
    Ok(BakeParams {
        grow,
        feather,
        range: window,
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
    asset_id: AssetKey,
) -> Result<SceneImage, AppError> {
    render_scene(
        state,
        RenderIdentity::from(ctx),
        ctx.immich.clone(),
        ctx.owner,
        asset_id,
    )
    .await
}

pub async fn generate(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(asset_id): Path<AssetKey>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    if !state.segment.enabled() {
        return Err(AppError::BadRequest(
            "segmentation is disabled on this server".into(),
        ));
    }
    let kind = parse_kind(&req.kind)?;
    if kind == ModelKind::Semantic && req.class.is_none() {
        return Err(AppError::BadRequest("semantic masks need a class".into()));
    }
    let params = bake_params(req.grow, req.feather, req.range.as_ref())?;

    let SceneImage {
        rgb8,
        width,
        height,
    } = scene_render(&state, &ctx, asset_id).await?;
    let result = state
        .segment
        .generate(kind, rgb8, width, height, params, req.class)
        .await
        .map_err(map_segment_err)?;

    let baked = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &result.bytes, width, height)
        .await?;
    let prob = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &result.prob, width, height)
        .await?;

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
    let params = bake_params(req.grow, req.feather, req.range.as_ref())?;
    let (meta, prob) = state
        .rasters
        .load(ctx.server_epoch, ctx.owner, &req.prob_raster_id)
        .await
        .map_err(|_| AppError::NotFound)?;

    let SceneImage {
        rgb8,
        width,
        height,
    } = scene_render(&state, &ctx, req.asset_id).await?;
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
        .await?;

    Ok(Json(RebakeResponse {
        raster_id: baked.raster_id,
        width,
        height,
    }))
}

pub async fn click(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(asset_id): Path<AssetKey>,
    Json(req): Json<ClickRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    if !state.segment.enabled() {
        return Err(AppError::BadRequest(
            "segmentation is disabled on this server".into(),
        ));
    }
    if req.points.is_empty() && req.bbox.is_none() {
        return Err(AppError::BadRequest("no points given".into()));
    }
    if req.points.len() > N_MAX_CLICK_POINTS {
        return Err(AppError::BadRequest(format!(
            "too many points: {} (max {N_MAX_CLICK_POINTS})",
            req.points.len()
        )));
    }
    if !req
        .points
        .iter()
        .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y))
    {
        return Err(AppError::BadRequest(
            "point coordinates must be within 0..1".into(),
        ));
    }
    let params = bake_params(req.grow, req.feather, None)?;
    let bbox = match req.bbox {
        Some(b) => Some(scene_box(&b)?),
        None => None,
    };

    let SceneImage {
        rgb8,
        width,
        height,
    } = scene_render(&state, &ctx, asset_id).await?;
    let base = match &req.base_raster_id {
        Some(id) => {
            let (meta, bytes) = state
                .rasters
                .load(ctx.server_epoch, ctx.owner, id)
                .await
                .map_err(|_| AppError::NotFound)?;
            if meta.width != width || meta.height != height {
                return Err(AppError::Conflict(
                    "shape raster does not match the current scene size".into(),
                ));
            }
            Some(bytes)
        }
        None => None,
    };
    let points: Vec<ClickPoint> = req
        .points
        .iter()
        .map(|p| ClickPoint {
            x: p.x * width as f32,
            y: p.y * height as f32,
            positive: p.positive,
        })
        .collect();

    let bbox = bbox.map(|b| BoxPrompt {
        x0: b.x0 * width as f32,
        y0: b.y0 * height as f32,
        x1: b.x1 * width as f32,
        y1: b.y1 * height as f32,
    });

    let key = EmbeddingKey {
        server_epoch: ctx.server_epoch,
        owner: ctx.owner,
        asset_id,
        width,
        height,
    };
    let result = state
        .segment
        .click(key, rgb8.clone(), points, bbox, params)
        .await
        .map_err(map_segment_err)?;

    let (baked_bytes, prob_bytes) = match base {
        Some(base) => {
            let prob = combine_coverage(&base, &result.prob, req.subtract);
            let baked = state
                .segment
                .rebake(prob.clone(), rgb8, width, height, params)
                .await
                .map_err(map_segment_err)?;
            (baked, prob)
        }
        None => (result.bytes, result.prob),
    };

    let baked = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &baked_bytes, width, height)
        .await?;
    let prob = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &prob_bytes, width, height)
        .await?;

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
