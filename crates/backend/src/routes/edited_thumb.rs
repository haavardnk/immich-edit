use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::edited_thumb::EditedThumbError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EditedThumbQuery {
    pub h: String,
    #[serde(default)]
    pub size: Option<u32>,
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<EditedThumbQuery>,
) -> Result<Response, AppError> {
    let size = q.size.unwrap_or(400).clamp(128, 1024);
    let record = state.edits.get(id).await.map_err(|e| {
        tracing::error!(error = %e, "edits store");
        AppError::Internal
    })?;
    let Some(record) = record else {
        return Err(AppError::NotFound);
    };
    let edits = record.manifest.to_edits().clamped();
    let bytes = state
        .edited_thumb
        .get_or_render(&state.render, id, edits, &q.h, size)
        .await
        .map_err(map_err)?;
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=31536000, immutable"),
    );
    if let Ok(etag) = HeaderValue::from_str(&format!("\"{}-{}\"", q.h, size)) {
        resp.headers_mut().insert(header::ETAG, etag);
    }
    Ok(resp.into_response())
}

fn map_err(err: EditedThumbError) -> AppError {
    match err {
        EditedThumbError::NotFound | EditedThumbError::HashMismatch => AppError::NotFound,
        EditedThumbError::Render(crate::services::render::RenderError::Upstream(u)) => u.into(),
        EditedThumbError::Render(crate::services::render::RenderError::Pipeline(p)) => {
            tracing::error!(error = %p, "edited thumb render");
            AppError::Internal
        }
        EditedThumbError::Io(e) => {
            tracing::error!(error = %e, "edited thumb io");
            AppError::Internal
        }
    }
}
