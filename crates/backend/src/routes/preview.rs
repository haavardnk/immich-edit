use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{OutputColorSpace, PreviewMode};
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::preview_meta::PreviewMeta;
use crate::services::render::{RenderError, RenderIdentity};
use crate::services::render_queue::RenderKey;
use crate::state::AppState;

const META_HEADER: &str = "x-preview-meta-id";
const MASK_PREVIEW_MAX_EDGE: u32 = 1400;

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default)]
    pub max: Option<u32>,
    #[serde(default)]
    pub clip: bool,
}

#[derive(Debug, Deserialize)]
pub struct LivePreviewBody {
    pub max_edge: Option<u32>,
    #[serde(default)]
    pub edits: Edits,
    #[serde(default)]
    pub preview_mode: PreviewMode,
    #[serde(default)]
    pub output_color_space: OutputColorSpace,
    #[serde(default)]
    pub gamut_warn: bool,
    #[serde(default)]
    pub clip_warn: bool,
}

pub async fn get_preview(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(q): Query<PreviewQuery>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let max_edge = clamp_max(state.config.preview_max_edge, q.max)?;
    let edits = state.edits.get_edits_or_default(ctx.owner, id).await?;
    let dcp_revision = state.render.dcp_revision().await.map_err(map_render_err)?;
    let etag = format!(
        "\"{}-{}-{}-{}-{}\"",
        edits.stable_hash(),
        max_edge,
        ctx.server_epoch,
        dcp_revision,
        q.clip as u8
    );
    let unchanged = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == etag));
    if unchanged {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        attach_validators(&mut resp, &etag);
        return Ok(resp);
    }
    let mut resp = render_to_response(
        &state,
        &ctx,
        id,
        edits,
        max_edge,
        PreviewMode::None,
        OutputColorSpace::SRgb,
        false,
        q.clip,
    )
    .await?;
    attach_validators(&mut resp, &etag);
    Ok(resp)
}

fn attach_validators(resp: &mut Response, etag: &str) {
    if let Ok(value) = HeaderValue::from_str(etag) {
        resp.headers_mut().insert(header::ETAG, value);
    }
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=0, must-revalidate"),
    );
}

pub async fn post_preview(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<LivePreviewBody>,
) -> Result<Response, AppError> {
    let max_edge = clamp_max(state.config.preview_max_edge, body.max_edge)?;
    let edits = body.edits.clamped();
    render_to_response(
        &state,
        &ctx,
        id,
        edits,
        max_edge,
        body.preview_mode,
        body.output_color_space,
        body.gamut_warn,
        body.clip_warn,
    )
    .await
}

pub async fn get_meta(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path((asset_id, meta_id)): Path<(AssetKey, Uuid)>,
) -> Result<Json<PreviewMeta>, AppError> {
    match state.preview_meta.get(meta_id).await {
        Some(meta) if meta.owner == ctx.owner && meta.asset_id == asset_id => Ok(Json(meta)),
        None => Err(AppError::NotFound),
        Some(_) => Err(AppError::NotFound),
    }
}

#[allow(clippy::too_many_arguments)]
async fn render_to_response(
    state: &AppState,
    ctx: &AuthCtx,
    asset_id: AssetKey,
    edits: Edits,
    max_edge: u32,
    preview_mode: PreviewMode,
    output_color_space: OutputColorSpace,
    gamut_warn: bool,
    clip_warn: bool,
) -> Result<Response, AppError> {
    let render = state.render.clone();
    let identity = RenderIdentity {
        owner: ctx.owner,
        server_epoch: ctx.server_epoch,
    };
    let key = RenderKey {
        owner: ctx.owner,
        server_epoch: ctx.server_epoch,
        asset_id,
    };
    let tracker = state.queue.tracker(key).await;
    let token = tracker.next();
    let max_edge = if matches!(preview_mode, PreviewMode::MaskWeight { .. }) {
        max_edge.min(MASK_PREVIEW_MAX_EDGE)
    } else {
        max_edge
    };
    let opts = raw_pipeline::frame::RenderOptions {
        max_edge,
        quality: false,
        output: raw_pipeline::frame::OutputFormat::Jpeg {
            quality: 85,
            subsampling: raw_pipeline::frame::JpegSubsampling::Chroma444,
        },
        output_color_space,
        preview_mode: preview_mode.clone(),
        gamut_warn,
        clip_warn,
        ..Default::default()
    };
    let work = render.render(
        identity,
        ctx.immich.clone(),
        asset_id.source(),
        edits,
        opts,
        Some(token),
    );
    let result = state.queue.enqueue::<_, _, RenderError>(key, work).await;
    let rendered = match result {
        Some(Ok(r)) => r,
        Some(Err(e)) => return Err(map_render_err(e)),
        None => {
            return Err(AppError::Superseded);
        }
    };

    let mut resp = Response::new(Body::from(rendered.bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    if matches!(preview_mode, PreviewMode::None) && !gamut_warn && !clip_warn {
        let meta = PreviewMeta {
            owner: ctx.owner,
            asset_id,
            width: rendered.width,
            height: rendered.height,
            source_w: rendered.source_w,
            source_h: rendered.source_h,
            renderer: rendered.renderer.clone(),
            histogram: rendered.histogram.clone(),
            linear_histogram: rendered.linear_histogram.clone(),
        };
        let meta_id = state.preview_meta.put(meta).await;
        resp.headers_mut().insert(
            HeaderName::from_static(META_HEADER),
            HeaderValue::from_str(&meta_id.to_string()).expect("uuid is valid header value"),
        );
    }
    Ok(resp.into_response())
}

fn clamp_max(default: u32, requested: Option<u32>) -> Result<u32, AppError> {
    let value = requested.unwrap_or(default);
    if !(64..=8192).contains(&value) {
        return Err(AppError::BadRequest(format!(
            "max_edge out of range: {value}"
        )));
    }
    Ok(value.min(default))
}

pub(crate) fn map_render_err(err: RenderError) -> AppError {
    match err {
        RenderError::Upstream(e) => e.into(),
        RenderError::Pipeline(raw_pipeline::PipelineError::Unsupported(msg)) => {
            AppError::UnsupportedFormat(msg)
        }
        RenderError::Pipeline(raw_pipeline::PipelineError::Cancelled) => AppError::Superseded,
        RenderError::Pipeline(_) => {
            tracing::error!(error = %err, "render pipeline");
            AppError::Internal
        }
        RenderError::Lut(m) => AppError::BadRequest(m),
        RenderError::Dcp(m) => AppError::BadRequest(m),
    }
}
