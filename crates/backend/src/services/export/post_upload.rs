use uuid::Uuid;

use crate::immich::dto::{AssetDetail, BulkIdResponse};
use crate::immich::{ImmichClient, ImmichError};
use crate::services::render::RenderIdentity;
use crate::state::AppState;

use super::replace::{Replacement, replace_original};
use super::{ExportToImmichBody, StackPrimary, immich_metadata};

pub(super) struct Finished {
    pub warnings: Vec<String>,
    pub original_trashed: bool,
}

pub(super) async fn finish_upload(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    original: &AssetDetail,
    body: &ExportToImmichBody,
    new_id: Uuid,
    upload_status: &str,
) -> Finished {
    let mut warnings = run_post_upload(
        state,
        identity,
        immich,
        original,
        body,
        new_id,
        upload_status,
    )
    .await;
    if !body.replace_original {
        return Finished {
            warnings,
            original_trashed: false,
        };
    }
    let replacement = Replacement {
        original,
        new_id,
        step_failed: !warnings.is_empty(),
        favorite_set: body.favorite,
    };
    let replaced = replace_original(state, identity, immich, replacement).await;
    if let Err(warning) = &replaced {
        warnings.push(warning.clone());
    }
    Finished {
        warnings,
        original_trashed: replaced.is_ok(),
    }
}

async fn run_post_upload(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    original: &AssetDetail,
    body: &ExportToImmichBody,
    new_id: Uuid,
    upload_status: &str,
) -> Vec<String> {
    let mut warnings: Vec<String> = Vec::new();
    let is_duplicate = upload_status.eq_ignore_ascii_case("duplicate");

    if body.favorite
        && is_duplicate
        && let Err(e) = immich
            .update_asset(new_id, &serde_json::json!({ "isFavorite": true }))
            .await
    {
        warnings.push(format!("Favorite failed: {}", e.short()));
    }

    if let Some(update) =
        immich_metadata::bulk_update(new_id, original.exif_info.as_ref(), body.params.metadata())
    {
        if !immich_metadata::await_extraction(immich, new_id).await {
            warnings.push("Metadata copied before Immich read the upload".into());
        }
        if let Err(e) = immich.update_assets(&update).await {
            warnings.push(format!("Metadata copy failed: {}", e.short()));
        }
    }

    for album_id in &body.album_ids {
        match immich.add_assets_to_album(*album_id, &[new_id]).await {
            Ok(items) => {
                for error in refusals(items) {
                    warnings.push(format!("Album {album_id} failed: {error}"));
                }
            }
            Err(e) => warnings.push(format!("Album {album_id} failed: {}", e.short())),
        }
    }

    for tag_id in &body.tag_ids {
        match immich.set_asset_tag(*tag_id, new_id, true).await {
            Ok(items) => {
                state
                    .tag_counts
                    .invalidate(identity.owner, identity.server_epoch, *tag_id)
                    .await;
                for error in refusals(items) {
                    warnings.push(format!("Tag {tag_id} failed: {error}"));
                }
            }
            Err(e) => warnings.push(format!("Tag {tag_id} failed: {}", e.short())),
        }
    }

    if body.stack_with_original {
        if original.owned_by_other(identity.owner) {
            warnings.push("Stacking skipped: the original belongs to another Immich user".into());
        } else if let Err(e) =
            stack_with_original(immich, original, new_id, body.stack_primary).await
        {
            warnings.push(format!("Stacking failed: {}", e.short()));
        }
    }

    warnings
}

fn refusals(items: Vec<BulkIdResponse>) -> impl Iterator<Item = String> {
    items
        .into_iter()
        .filter(|item| !item.success)
        .map(|item| item.error.unwrap_or_else(|| "unknown".into()))
        .filter(|error| error != "duplicate")
}

async fn stack_with_original(
    immich: &ImmichClient,
    original: &AssetDetail,
    new_id: Uuid,
    primary: StackPrimary,
) -> Result<(), ImmichError> {
    let mut ids: Vec<Uuid> = vec![new_id, original.id.source()];
    if let Some(stack_id) = original.current_stack_id()
        && let Ok(stack) = immich.get_stack(stack_id).await
    {
        for a in stack.assets {
            if !ids.contains(&a.id.source()) {
                ids.push(a.id.source());
            }
        }
    }
    let primary_id = match primary {
        StackPrimary::Edited => new_id,
        StackPrimary::Original => original.id.source(),
    };
    if let Some(pos) = ids.iter().position(|i| *i == primary_id) {
        ids.swap(0, pos);
    }
    let created = immich.create_stack(&ids).await?;
    if created.primary_asset_id != primary_id {
        immich.update_stack_primary(created.id, primary_id).await?;
    }
    Ok(())
}
