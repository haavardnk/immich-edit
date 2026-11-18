use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::CopyFlags;
use crate::immich::dto::AssetDetail;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::edits_store::ExportOutputKey;
use crate::services::render::RenderIdentity;
use crate::services::trash;
use crate::state::AppState;

use super::conflict::is_missing;

#[derive(Debug, Clone, Copy)]
pub(super) struct Replacement {
    pub new_id: Uuid,
    pub step_failed: bool,
    pub favorite_set: bool,
}

enum Kept {
    SameAsset,
    Because(String),
}

pub(super) async fn replace_original(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    original: &AssetDetail,
    replacement: Replacement,
) -> Result<(), String> {
    let outcome = match guard(identity, original, replacement) {
        Ok(()) => swap(state, identity, immich, original, replacement).await,
        Err(kept) => Err(kept),
    };
    outcome.map_err(|kept| match kept {
        Kept::SameAsset => original_kept("Immich matched the upload to the original"),
        Kept::Because(reason) => original_kept(&reason),
    })
}

pub(super) async fn replace_previous(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    output: ExportOutputKey<'_>,
    previous: Uuid,
    replacement: Replacement,
) -> Result<Option<Uuid>, String> {
    if previous == replacement.new_id {
        return Ok(None);
    }
    let target = match immich.asset(previous).await {
        Ok(target) => target,
        Err(err) if is_missing(&err) => return Ok(None),
        Err(err) => return Err(previous_kept(&failed("reading it", &err))),
    };
    let outcome = match guard(identity, &target, replacement) {
        Ok(()) => match still_used(state, output, previous).await {
            Ok(()) => swap(state, identity, immich, &target, replacement).await,
            Err(kept) => Err(kept),
        },
        Err(kept) => Err(kept),
    };
    match outcome {
        Ok(()) => Ok(Some(previous)),
        Err(Kept::SameAsset) => Ok(None),
        Err(Kept::Because(reason)) => Err(previous_kept(&reason)),
    }
}

fn guard(
    identity: RenderIdentity,
    target: &AssetDetail,
    replacement: Replacement,
) -> Result<(), Kept> {
    if replacement.step_failed {
        return Err(because("an earlier step failed"));
    }
    if target.owned_by_other(identity.owner) {
        return Err(because("it belongs to another Immich user"));
    }
    if target.id.source() == replacement.new_id {
        return Err(Kept::SameAsset);
    }
    Ok(())
}

async fn still_used(
    state: &AppState,
    output: ExportOutputKey<'_>,
    previous: Uuid,
) -> Result<(), Kept> {
    let unreadable = |_| because("checking its other uses failed");
    if state
        .edits
        .export_output_shared(output, previous)
        .await
        .map_err(unreadable)?
    {
        return Err(because("another export still uses it"));
    }
    if state
        .edits
        .has_edits_for_source(output.owner, previous)
        .await
        .map_err(unreadable)?
    {
        return Err(because("it has its own edits"));
    }
    Ok(())
}

async fn swap(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    target: &AssetDetail,
    replacement: Replacement,
) -> Result<(), Kept> {
    let new_id = replacement.new_id;
    let source = target.id.source();
    trash::ensure_enabled(immich).await.map_err(refusal)?;
    let edit = immich
        .asset(new_id)
        .await
        .map_err(|e| Kept::Because(failed("reading the upload", &e)))?;
    if edit.is_trashed {
        return Err(because("the matching upload is in the Immich trash"));
    }
    let stack = target.current_stack_id();
    let flags = CopyFlags {
        stack: stack.is_none() || edit.current_stack_id() != stack,
        favorite: !replacement.favorite_set,
    };
    immich
        .copy_asset(source, new_id, flags)
        .await
        .map_err(|e| Kept::Because(failed("copying its albums and links", &e)))?;
    if let Some(stack) = target
        .stack
        .as_ref()
        .filter(|s| s.primary_asset_id == source)
    {
        immich
            .update_stack_primary(stack.id, new_id)
            .await
            .map_err(|e| Kept::Because(failed("putting the upload on top of its stack", &e)))?;
    }
    trash::discard(state, identity, immich, &[source])
        .await
        .map_err(refusal)
}

fn refusal(err: AppError) -> Kept {
    match err {
        AppError::TrashDisabled => because("the Immich trash is turned off"),
        AppError::UpstreamRejected(message) => Kept::Because(message),
        other => Kept::Because(other.to_string()),
    }
}

fn because(reason: &str) -> Kept {
    Kept::Because(reason.into())
}

fn failed(step: &str, err: &ImmichError) -> String {
    format!("{step} failed: {}", err.short())
}

fn original_kept(reason: &str) -> String {
    format!("Original kept: {reason}")
}

fn previous_kept(reason: &str) -> String {
    format!("Previous export kept: {reason}")
}
