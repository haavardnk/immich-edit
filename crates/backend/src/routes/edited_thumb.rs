use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::headers::{self, attach_validators, etag_matches};
use crate::services::edited_thumb::ThumbRequest;
use crate::services::render::{AssetRevision, RenderIdentity};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EditedThumbQuery {
    pub h: String,
    #[serde(default)]
    pub r: Option<String>,
    #[serde(default)]
    pub size: Option<u32>,
}

fn cache_control(q: &EditedThumbQuery, revision: &AssetRevision) -> &'static str {
    let pinned = revision.settled() && q.r.as_deref() == Some(revision.to_string().as_str());
    if pinned {
        headers::CACHE_IMMUTABLE
    } else {
        headers::CACHE_REVALIDATE
    }
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(q): Query<EditedThumbQuery>,
    request_headers: HeaderMap,
) -> Result<Response, AppError> {
    let size = q.size.unwrap_or(400).clamp(128, 1024);
    let record = state.edits.get(ctx.owner, id).await?;
    let Some(record) = record else {
        return Err(AppError::NotFound);
    };
    let edits = record.manifest.to_edits().clamped();
    if edits.stable_hash() != q.h {
        return Err(AppError::NotFound);
    }
    let identity = RenderIdentity::from(&ctx);
    let revision = state
        .render
        .asset_revision(identity, id.source(), &edits)
        .await?;
    let etag = headers::etag(&format!("{}-{}-{}", q.h, size, revision))?;
    let cache = cache_control(&q, &revision);
    if etag_matches(&request_headers, &etag) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        attach_validators(&mut resp, etag, cache);
        return Ok(resp);
    }
    let thumb = state
        .edited_thumb
        .get_or_render(
            &state.render,
            identity,
            ctx.immich.clone(),
            ThumbRequest {
                asset_id: id,
                edits,
                revision: &revision,
                size,
            },
        )
        .await
        .map_err(AppError::from)?;
    let cache = if thumb.current {
        cache
    } else {
        headers::CACHE_REVALIDATE
    };
    let mut resp = Response::new(Body::from(thumb.bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    attach_validators(&mut resp, etag, cache);
    Ok(resp)
}
