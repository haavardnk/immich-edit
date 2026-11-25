use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use raw_pipeline::edit_manifest::EditManifest;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::{EditWrite, WriteOutcome};
use crate::state::AppState;

pub struct Precondition<'a> {
    pub expected: &'a str,
    pub amend: bool,
}

pub async fn write_manifest(
    state: &AppState,
    ctx: &AuthCtx,
    id: AssetKey,
    manifest: EditManifest,
    action: Option<&str>,
    precondition: Option<Precondition<'_>>,
) -> Result<Response, AppError> {
    let asset = ctx.immich.asset(id.source()).await?;
    let write = EditWrite {
        manifest,
        immich_updated_at: asset.updated_at,
        immich_checksum: asset.checksum,
        action,
    };
    let Some(Precondition { expected, amend }) = precondition else {
        let saved = state.edits.put(ctx.owner, id, write).await?;
        return Ok(Json(saved).into_response());
    };
    let outcome = if amend {
        state
            .edits
            .amend_if_match(ctx.owner, id, expected, write)
            .await?
    } else {
        state
            .edits
            .put_if_match(ctx.owner, id, expected, write)
            .await?
    };
    match outcome {
        WriteOutcome::Written(saved) => Ok(Json(saved).into_response()),
        WriteOutcome::Conflict(current) => {
            Ok((StatusCode::CONFLICT, Json(current)).into_response())
        }
    }
}
