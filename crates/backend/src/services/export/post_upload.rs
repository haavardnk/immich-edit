use uuid::Uuid;

use crate::immich::ImmichClient;
use crate::immich::dto::{AssetDetail, BulkIdResponse};
use crate::services::edits_store::ExportOutputKey;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

use super::replace::{Replacement, replace_original, replace_previous};
use super::stack::stack_with_original;
use super::{ExportToImmichBody, immich_metadata};

pub(super) struct Finished {
    pub warnings: Vec<String>,
    pub original_trashed: bool,
    pub replaced_asset_id: Option<Uuid>,
}

pub(super) struct Uploaded<'a> {
    pub original: &'a AssetDetail,
    pub body: &'a ExportToImmichBody,
    pub id: Uuid,
    pub status: &'a str,
    pub variant: i64,
    pub output: ExportOutputKey<'a>,
    pub previous: Option<Uuid>,
}

pub(super) async fn finish_upload(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    upload: Uploaded<'_>,
) -> Finished {
    let mut warnings = run_post_upload(state, identity, immich, &upload).await;
    let replacement = Replacement {
        new_id: upload.id,
        step_failed: !warnings.is_empty(),
        favorite_set: upload.body.favorite,
    };
    let mut replaced_asset_id = None;
    if let Some(previous) = upload.previous
        && upload.id != upload.output.asset_id.source()
    {
        let replaced = replace_previous(
            state,
            identity,
            immich,
            upload.output,
            previous,
            replacement,
        )
        .await;
        replaced_asset_id = settle(&mut warnings, replaced).flatten();
    }
    let mut original_trashed = false;
    if upload.body.replace_original {
        let replaced =
            replace_original(state, identity, immich, upload.original, replacement).await;
        original_trashed = settle(&mut warnings, replaced).is_some();
    }
    Finished {
        warnings,
        original_trashed,
        replaced_asset_id,
    }
}

fn settle<T>(warnings: &mut Vec<String>, outcome: Result<T, String>) -> Option<T> {
    match outcome {
        Ok(value) => Some(value),
        Err(warning) => {
            warnings.push(warning);
            None
        }
    }
}

async fn run_post_upload(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    upload: &Uploaded<'_>,
) -> Vec<String> {
    let Uploaded {
        original,
        body,
        id: new_id,
        status: upload_status,
        ..
    } = *upload;
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
        } else if let Err(e) = stack_with_original(state, immich, identity.owner, upload).await {
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
