use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::CopyFlags;
use crate::immich::dto::AssetDetail;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::render::RenderIdentity;
use crate::services::trash;
use crate::state::AppState;

pub(super) struct Replacement<'a> {
    pub original: &'a AssetDetail,
    pub new_id: Uuid,
    pub step_failed: bool,
    pub favorite_set: bool,
}

pub(super) async fn replace_original(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    replacement: Replacement<'_>,
) -> Result<(), String> {
    let Replacement {
        original,
        new_id,
        step_failed,
        favorite_set,
    } = replacement;
    let source = original.id.source();
    if step_failed {
        return Err(kept("an earlier step failed"));
    }
    if original.owned_by_other(identity.owner) {
        return Err(kept("it belongs to another Immich user"));
    }
    if source == new_id {
        return Err(kept("Immich matched the upload to the original"));
    }
    trash::ensure_enabled(immich).await.map_err(refusal)?;
    let edit = immich
        .asset(new_id)
        .await
        .map_err(|e| failed("reading the upload", &e))?;
    if edit.is_trashed {
        return Err(kept("the matching upload is in the Immich trash"));
    }
    let stack = original.current_stack_id();
    let flags = CopyFlags {
        stack: stack.is_none() || edit.current_stack_id() != stack,
        favorite: !favorite_set,
    };
    immich
        .copy_asset(source, new_id, flags)
        .await
        .map_err(|e| failed("copying its albums and links", &e))?;
    if let Some(stack) = original
        .stack
        .as_ref()
        .filter(|s| s.primary_asset_id == source)
    {
        immich
            .update_stack_primary(stack.id, new_id)
            .await
            .map_err(|e| failed("putting the upload on top of its stack", &e))?;
    }
    trash::discard(state, identity, immich, &[source])
        .await
        .map_err(refusal)
}

fn refusal(err: AppError) -> String {
    match err {
        AppError::TrashDisabled => kept("the Immich trash is turned off"),
        AppError::UpstreamRejected(message) => kept(&message),
        other => kept(&other.to_string()),
    }
}

fn failed(step: &str, err: &ImmichError) -> String {
    kept(&format!("{step} failed: {}", err.short()))
}

fn kept(reason: &str) -> String {
    format!("Original kept: {reason}")
}
