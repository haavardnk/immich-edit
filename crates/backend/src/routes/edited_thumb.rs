use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::routes::headers;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EditedThumbQuery {
    pub h: String,
    #[serde(default)]
    pub size: Option<u32>,
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
    Query(q): Query<EditedThumbQuery>,
) -> Result<Response, AppError> {
    let size = q.size.unwrap_or(400).clamp(128, 1024);
    let record = state.edits.get(ctx.owner, id).await?;
    let Some(record) = record else {
        return Err(AppError::NotFound);
    };
    let edits = record.manifest.to_edits().clamped();
    let bytes = state
        .edited_thumb
        .get_or_render(
            &state.render,
            RenderIdentity::from(&ctx),
            ctx.immich.clone(),
            id,
            edits,
            &q.h,
            size,
        )
        .await
        .map_err(AppError::from)?;
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(headers::CACHE_IMMUTABLE),
    );
    resp.headers_mut()
        .insert(header::ETAG, headers::etag(&format!("{}-{}", q.h, size))?);
    Ok(resp.into_response())
}
