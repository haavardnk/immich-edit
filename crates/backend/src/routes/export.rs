use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::export::{
    self, ExportBody, ExportImmichRequest, ExportToImmichResult, NameContext, NameTemplate, Seq,
    capture_date,
};
use crate::services::render::RenderIdentity;
use crate::services::render_queue::RenderPriority;
use crate::state::AppState;

pub use crate::services::export::{
    ColorSpaceOpt, ExportParams, ExportToImmichBody, StackPrimary, hash_request, resolve_filename,
};

pub async fn get_export(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(params): Query<ExportParams>,
) -> Result<Response, AppError> {
    let edits = state.edits.get_edits_or_default(ctx.owner, id).await?;
    download(&state, &ctx, id, edits, &params).await
}

pub async fn post_export(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Json(body): Json<ExportBody>,
) -> Result<Response, AppError> {
    download(&state, &ctx, id, body.edits.clamped(), &body.params).await
}

async fn download(
    state: &AppState,
    ctx: &AuthCtx,
    id: AssetKey,
    edits: raw_pipeline::edits::Edits,
    params: &ExportParams,
) -> Result<Response, AppError> {
    let template = NameTemplate::parse(params.filename_template.as_deref())?;
    let (rendered, asset) = tokio::join!(
        export::render_export(
            state,
            RenderIdentity::from(ctx),
            &ctx.immich,
            id,
            edits,
            params,
            RenderPriority::Interactive,
        ),
        ctx.immich.asset(id.source()),
    );
    let (bytes, output) = rendered?;
    let asset = asset?;
    let stem = template.render(&NameContext {
        original: &asset.original_file_name,
        date: capture_date(&asset),
        seq: Seq::SINGLE,
    });
    let filename = format!("{stem}.{}", output.extension());
    Ok(download_response(&filename, bytes, output))
}

fn content_disposition(filename: &str) -> HeaderValue {
    let ascii: String = filename
        .chars()
        .map(|c| {
            if (c.is_ascii_graphic() && c != '"' && c != '\\') || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let encoded: String = filename
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&b) {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    HeaderValue::from_str(&format!(
        "attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}"
    ))
    .unwrap_or(HeaderValue::from_static("attachment"))
}

fn download_response(
    filename: &str,
    bytes: bytes::Bytes,
    output: raw_pipeline::frame::OutputFormat,
) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(output.content_type()),
    );
    resp.headers_mut()
        .insert(header::CONTENT_DISPOSITION, content_disposition(filename));
    resp.into_response()
}

pub async fn post_export_immich(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    headers: HeaderMap,
    Json(body): Json<ExportToImmichBody>,
) -> Result<Json<ExportToImmichResult>, AppError> {
    let idem_key = idempotency_key(&headers)?;
    let result = export::export_to_immich(
        &state,
        &ctx.immich,
        ctx.owner,
        ExportImmichRequest {
            asset_id: id,
            server_epoch: ctx.server_epoch,
            body: &body,
            idempotency_key: idem_key,
            priority: RenderPriority::Interactive,
            seq: Seq::SINGLE,
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
