use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::services::edit_merge::legacy::{LegacySections, merge_legacy};
use crate::services::edit_merge::{MergeSections, merge_edits};
use crate::services::edits_store::EditWrite;
use crate::services::job_runner::{ItemOutcome, JobItemError};
use crate::services::job_store::JobRecord;
use crate::state::AppState;

pub const PASTE_EDITS_KIND: &str = "paste_edits";

#[derive(Debug, Deserialize)]
pub struct PasteEditsParams {
    pub manifest: EditManifest,
    #[serde(default)]
    pub tools: Option<MergeSections>,
    #[serde(default)]
    pub sections: Option<LegacySections>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteSelection {
    Tools(MergeSections),
    Legacy(LegacySections),
}

impl PasteSelection {
    pub fn merge(self, current: Edits, incoming: Edits) -> Edits {
        match self {
            Self::Tools(tools) => merge_edits(current, incoming, tools),
            Self::Legacy(sections) => merge_legacy(current, incoming, sections),
        }
    }

    pub fn pastes_lens(self) -> bool {
        match self {
            Self::Tools(tools) => tools.lens,
            Self::Legacy(sections) => sections.lens,
        }
    }
}

impl PasteEditsParams {
    pub fn selection(&self) -> PasteSelection {
        match (self.tools, self.sections) {
            (Some(tools), _) => PasteSelection::Tools(tools),
            (None, Some(sections)) => PasteSelection::Legacy(sections),
            (None, None) => PasteSelection::Tools(MergeSections::paste_default()),
        }
    }
}

pub async fn run_paste_edits_item(
    state: &AppState,
    job: &JobRecord,
    asset_id: AssetKey,
) -> ItemOutcome {
    let params: PasteEditsParams = serde_json::from_value(job.params.clone())
        .map_err(|e| JobItemError::msg(format!("invalid paste edits params: {e}")))?;
    let current = state
        .edits
        .get_edits_or_default(job.user_id, asset_id)
        .await?;
    let selection = params.selection();
    let mut merged = selection.merge(current, params.manifest.to_edits());
    let immich = crate::services::export::job_immich(state, job).await?;
    let asset = immich.asset(asset_id.source()).await?;
    if selection.pastes_lens() {
        merged.lens = crate::lens_profile::reproject_lens(merged.lens, asset.exif_info.as_ref());
    }
    let manifest = EditManifest::from_edits(&merged);
    let saved = state
        .edits
        .put(
            job.user_id,
            asset_id,
            EditWrite {
                manifest,
                immich_updated_at: asset.updated_at,
                immich_checksum: asset.checksum,
                action: Some("Paste edits"),
            },
        )
        .await?;
    Ok(serde_json::json!({
        "hash": saved.hash,
        "updated_at": saved.updated_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(extra: serde_json::Value) -> PasteEditsParams {
        let mut value = serde_json::json!({ "manifest": { "schema_version": 1, "ops": {} } });
        if let (Some(target), Some(source)) = (value.as_object_mut(), extra.as_object()) {
            target.extend(source.clone());
        }
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn per_tool_params_merge_only_the_checked_tools() {
        let selection =
            params(serde_json::json!({ "tools": { "hsl": true, "lens": true } })).selection();
        let PasteSelection::Tools(tools) = selection else {
            panic!("expected per-tool selection, got {selection:?}");
        };
        assert!(tools.hsl && tools.lens);
        assert!(!tools.curves && !tools.bw && !tools.geometry);
        assert!(selection.pastes_lens());
    }

    #[test]
    fn a_queued_job_from_an_older_client_keeps_its_section_merge() {
        let selection = params(serde_json::json!({
            "sections": { "basic": false, "tone": true, "color": false, "detail": true,
                "effects": false, "lens": true, "geometry": false, "masks": false, "retouch": true }
        }))
        .selection();
        assert_eq!(
            selection,
            PasteSelection::Legacy(LegacySections {
                tone: true,
                detail: true,
                lens: true,
                retouch: true,
                ..LegacySections::default()
            })
        );
        assert!(selection.pastes_lens());
        let mut incoming = Edits::default();
        incoming.basic.exposure_ev = 1.0;
        incoming.tone.highlights = -30.0;
        let merged = selection.merge(Edits::default(), incoming);
        assert_eq!(merged.basic.exposure_ev, 0.0);
        assert_eq!(merged.tone.highlights, -30.0);
    }

    #[test]
    fn per_tool_params_win_over_legacy_sections() {
        let selection = params(serde_json::json!({
            "tools": { "hsl": true }, "sections": { "basic": true }
        }))
        .selection();
        assert!(matches!(selection, PasteSelection::Tools(tools) if tools.hsl && !tools.tone));
    }

    #[test]
    fn params_without_a_selection_paste_the_default_tools() {
        assert_eq!(
            params(serde_json::json!({})).selection(),
            PasteSelection::Tools(MergeSections::paste_default())
        );
    }
}
