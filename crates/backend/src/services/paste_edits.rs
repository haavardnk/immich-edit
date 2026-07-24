use raw_pipeline::edit_manifest::EditManifest;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::edit_merge::{MergeSections, merge_edits};
use crate::services::job_runner::ItemOutcome;
use crate::services::job_store::JobRecord;
use crate::state::AppState;

pub const PASTE_EDITS_KIND: &str = "paste_edits";

#[derive(Debug, Deserialize)]
pub struct PasteEditsParams {
    pub manifest: EditManifest,
    #[serde(default = "MergeSections::paste_default")]
    pub sections: MergeSections,
}

pub async fn run_paste_edits_item(
    state: &AppState,
    job: &JobRecord,
    asset_id: Uuid,
) -> ItemOutcome {
    let params: PasteEditsParams = serde_json::from_value(job.params.clone())
        .map_err(|e| format!("invalid paste edits params: {e}"))?;
    let current = state
        .edits
        .get_edits_or_default(job.user_id, asset_id)
        .await
        .map_err(|e| e.to_string())?;
    let mut merged = merge_edits(current, params.manifest.to_edits(), params.sections);
    let immich = crate::services::export::job_immich(state, job).await?;
    let asset = immich.asset(asset_id).await.map_err(|e| e.to_string())?;
    if params.sections.lens {
        merged.lens = crate::lens_profile::reproject_lens(merged.lens, asset.exif_info.as_ref());
    }
    let manifest = EditManifest::from_edits(&merged);
    let saved = state
        .edits
        .put(
            job.user_id,
            asset_id,
            manifest,
            asset.updated_at,
            asset.checksum,
            Some("Paste edits"),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "hash": saved.hash,
        "updated_at": saved.updated_at,
    }))
}
