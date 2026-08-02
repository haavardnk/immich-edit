use axum::Json;
use axum::extract::{Path, State};
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use segment::{BakeParams, ClickPoint, ModelKind, RangeWindow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::preview::map_render_err;
use crate::services::embedding_cache::EmbeddingKey;
use crate::services::render::RenderIdentity;
use crate::services::segment::SegmentServiceError;
use crate::state::AppState;

const MAX_REFINE_PX: f32 = 128.0;
const MAX_POINTS: usize = 32;

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
    pub points: Vec<ClickPointBody>,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub feather: f32,
    #[serde(default)]
    pub base_raster_id: Option<String>,
    #[serde(default)]
    pub subtract: bool,
}

fn combine_coverage(base: &[u8], patch: &[u8], subtract: bool) -> Vec<u8> {
    base.iter()
        .zip(patch)
        .map(|(b, p)| {
            if subtract {
                ((*b as u16 * (255 - *p) as u16 + 127) / 255) as u8
            } else {
                (*b).max(*p)
            }
        })
        .collect()
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
    pub asset_id: Uuid,
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
    if kind == ModelKind::Semantic && req.class.is_none() {
        return Err(AppError::BadRequest("semantic masks need a class".into()));
    }
    let params = bake_params(req.grow, req.feather, req.range.as_ref())?;

    let (rgb8, width, height) = scene_render(&state, &ctx, asset_id).await?;
    let result = state
        .segment
        .generate(kind, rgb8, width, height, params, req.class)
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
    let params = bake_params(req.grow, req.feather, req.range.as_ref())?;
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

pub async fn click(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(asset_id): Path<Uuid>,
    Json(req): Json<ClickRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    if !state.segment.enabled() {
        return Err(AppError::BadRequest(
            "segmentation is disabled on this server".into(),
        ));
    }
    if req.points.is_empty() {
        return Err(AppError::BadRequest("no points given".into()));
    }
    if req.points.len() > MAX_POINTS {
        return Err(AppError::BadRequest(format!(
            "too many points: {} (max {MAX_POINTS})",
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

    let (rgb8, width, height) = scene_render(&state, &ctx, asset_id).await?;
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

    let key = EmbeddingKey {
        server_epoch: ctx.server_epoch,
        owner: ctx.owner,
        asset_id,
        width,
        height,
    };
    let result = state
        .segment
        .click(key, rgb8.clone(), points, params)
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
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "raster store");
            AppError::Internal
        })?;
    let prob = state
        .rasters
        .store(ctx.server_epoch, ctx.owner, &prob_bytes, width, height)
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

#[cfg(test)]
mod tests {
    use super::combine_coverage;

    #[test]
    fn adding_keeps_the_strongest_coverage() {
        let out = combine_coverage(&[0, 128, 255], &[64, 32, 0], false);
        assert_eq!(out, vec![64, 128, 255]);
    }

    #[test]
    fn subtracting_carves_the_patch_out() {
        let out = combine_coverage(&[255, 255, 128], &[255, 0, 128], true);
        assert_eq!(out, vec![0, 255, 64]);
    }
}
