use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edits::Edits;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::render::RenderError;
use crate::state::AppState;

const EXPORT_MAX_EDGE: u32 = 8192;

#[derive(Debug, Deserialize)]
pub struct ExportBody {
    #[serde(default)]
    pub edits: Edits,
}

pub async fn get_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let edits = state.edits.get_edits_or_default(id).await.map_err(|e| {
        tracing::error!(error = %e, "edits store");
        AppError::Internal
    })?;
    export(state, id, edits).await
}

pub async fn post_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExportBody>,
) -> Result<Response, AppError> {
    export(state, id, body.edits.clamped()).await
}

async fn export(state: AppState, id: Uuid, edits: Edits) -> Result<Response, AppError> {
    let rendered = state
        .render
        .render(id, edits, EXPORT_MAX_EDGE)
        .await
        .map_err(map_render_err)?;

    let mut resp = Response::new(Body::from(rendered.jpeg));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{id}.jpg\"")).unwrap(),
    );
    Ok(resp.into_response())
}

fn map_render_err(err: RenderError) -> AppError {
    match err {
        RenderError::Upstream(e) => e.into(),
        RenderError::Pipeline(e) => {
            tracing::error!(error = %e, "export render");
            AppError::Internal
        }
    }
}
