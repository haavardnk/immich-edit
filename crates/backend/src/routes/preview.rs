use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::PreviewMode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::preview_meta::PreviewMeta;
use crate::services::render::RenderError;
use crate::state::AppState;

const META_HEADER: &str = "x-preview-meta-id";

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default)]
    pub max: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct LivePreviewBody {
    pub max_edge: Option<u32>,
    #[serde(default)]
    pub edits: Edits,
    #[serde(default)]
    pub preview_mode: PreviewMode,
}

pub async fn get_preview(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<PreviewQuery>,
) -> Result<Response, AppError> {
    let max_edge = clamp_max(state.config.preview_max_edge, q.max)?;
    let edits = state.edits.get_edits_or_default(id).await.map_err(|e| {
        tracing::error!(error = %e, "edits store");
        AppError::Internal
    })?;
    render_to_response(&state, id, edits, max_edge, PreviewMode::None).await
}

pub async fn post_preview(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<LivePreviewBody>,
) -> Result<Response, AppError> {
    let max_edge = clamp_max(state.config.preview_max_edge, body.max_edge)?;
    let edits = body.edits.clamped();
    render_to_response(&state, id, edits, max_edge, body.preview_mode).await
}

pub async fn get_meta(
    State(state): State<AppState>,
    Path((_id, meta_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<PreviewMeta>, AppError> {
    match state.preview_meta.get(meta_id).await {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound),
    }
}

async fn render_to_response(
    state: &AppState,
    asset_id: Uuid,
    edits: Edits,
    max_edge: u32,
    preview_mode: PreviewMode,
) -> Result<Response, AppError> {
    let render = state.render.clone();
    let tracker = state.queue.tracker(asset_id).await;
    let token = tracker.next();
    let opts = raw_pipeline::frame::RenderOptions {
        max_edge,
        quality: false,
        output: raw_pipeline::frame::OutputFormat::Jpeg { quality: 85 },
        preview_mode: preview_mode.clone(),
        ..Default::default()
    };
    let work = render.render(asset_id, edits, opts, Some(token));
    let result = state
        .queue
        .enqueue::<_, _, RenderError>(asset_id, work)
        .await;
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
    if matches!(preview_mode, PreviewMode::None) {
        let meta = PreviewMeta {
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

fn map_render_err(err: RenderError) -> AppError {
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
    }
}

#[allow(dead_code)]
fn _used(_: StatusCode) {}
