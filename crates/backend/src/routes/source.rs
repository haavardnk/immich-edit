use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RenderOptions;
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::preview::{attach_validators, clamp_max, etag_matches};
use crate::services::render::{RenderError, RenderIdentity};
use crate::services::render_queue::{CancelOnDrop, RenderKey, RenderLane};
use crate::state::AppState;

pub const SOURCE_CONTENT_TYPE: &str = "application/vnd.immich-edit.source";
const DCP_HEADER: &str = "x-source-dcp";

#[derive(Debug, Deserialize)]
pub struct SourceBody {
    pub max_edge: Option<u32>,
    #[serde(default)]
    pub edits: Edits,
}

pub async fn post_source(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    headers: HeaderMap,
    Json(body): Json<SourceBody>,
) -> Result<Response, AppError> {
    let max_edge = clamp_max(state.config.preview_max_edge, body.max_edge)?;
    let edits = body.edits.clamped().sensor_stage();
    let dcp_revision = state.render.dcp_revision().await?;
    let etag = format!(
        "\"{}-{}-{}-{}\"",
        edits.stable_hash(),
        max_edge,
        ctx.server_epoch,
        dcp_revision
    );
    if etag_matches(&headers, &etag) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        attach_validators(&mut resp, &etag);
        return Ok(resp);
    }
    let key = RenderKey {
        owner: ctx.owner,
        server_epoch: ctx.server_epoch,
        asset_id: id,
        lane: RenderLane::Source,
    };
    let token = state.queue.tracker(key).await.next();
    let opts = RenderOptions {
        max_edge,
        ..Default::default()
    };
    let work = state.render.source(
        RenderIdentity::from(&ctx),
        ctx.immich.clone(),
        id.source(),
        edits,
        opts,
        Some(token.clone()),
    );
    let guard = CancelOnDrop::new(token);
    let result = state.queue.enqueue::<_, _, RenderError>(key, work).await;
    guard.disarm();
    let source = match result {
        Some(Ok(source)) => source,
        Some(Err(e)) => return Err(e.into()),
        None => return Err(AppError::Superseded),
    };
    let mut resp = Response::new(Body::from(source.bytes));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(SOURCE_CONTENT_TYPE),
    );
    if let Some(value) = source.dcp_id.and_then(|id| HeaderValue::from_str(&id).ok()) {
        resp.headers_mut()
            .insert(HeaderName::from_static(DCP_HEADER), value);
    }
    attach_validators(&mut resp, &etag);
    Ok(resp)
}
