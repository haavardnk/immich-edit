use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::export::{self, ExportBody, ExportImmichRequest, ExportToImmichResult};
use crate::state::AppState;

pub use crate::services::export::{
    ExportParams, ExportToImmichBody, StackPrimary, hash_request, resolve_filename,
};

pub async fn get_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ExportParams>,
) -> Result<Response, AppError> {
    let edits = state.edits.get_edits_or_default(id).await.map_err(|e| {
        tracing::error!(error = %e, "edits store");
        AppError::Internal
    })?;
    let (bytes, output) = export::render_export(&state, id, edits, &params).await?;
    Ok(download_response(id, bytes, output))
}

pub async fn post_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExportBody>,
) -> Result<Response, AppError> {
    let (bytes, output) =
        export::render_export(&state, id, body.edits.clamped(), &body.params).await?;
    Ok(download_response(id, bytes, output))
}

fn download_response(
    id: Uuid,
    bytes: bytes::Bytes,
    output: raw_pipeline::frame::OutputFormat,
) -> Response {
    let content_type = output.content_type();
    let extension = output.extension();
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{id}.{extension}\"")).unwrap(),
    );
    resp.into_response()
}

pub async fn post_export_immich(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<ExportToImmichBody>,
) -> Result<Json<ExportToImmichResult>, AppError> {
    let idem_key = idempotency_key(&headers)?;
    let result = export::export_to_immich(
        &state,
        ExportImmichRequest {
            asset_id: id,
            body: &body,
            idempotency_key: idem_key,
            device_asset_id: None,
        },
    )
    .await?;
    Ok(Json(result))
}

fn idempotency_key(headers: &HeaderMap) -> Result<Option<String>, AppError> {
    let Some(v) = headers.get("idempotency-key") else {
        return Ok(None);
    };
    let s = v
        .to_str()
        .map_err(|_| AppError::BadRequest("invalid Idempotency-Key header".into()))?
        .trim();
    if s.is_empty() {
        return Ok(None);
    }
    if s.len() > 128 || !s.chars().all(|c| c.is_ascii_graphic()) {
        return Err(AppError::BadRequest(
            "invalid Idempotency-Key header".into(),
        ));
    }
    Ok(Some(s.to_string()))
}
