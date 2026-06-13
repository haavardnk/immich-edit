use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::job_runner::ItemOutcome;
use crate::services::job_store::JobRecord;
use crate::state::AppState;

pub const APPLY_PRESET_KIND: &str = "apply_preset";

#[derive(Debug, Deserialize)]
pub struct ApplyPresetParams {
    pub preset_id: Uuid,
    #[serde(default)]
    pub include_geometry: bool,
    #[serde(default)]
    pub include_masks: bool,
    #[serde(default)]
    pub include_output: bool,
}

pub fn merge_preset(current: Edits, preset: Edits, params: &ApplyPresetParams) -> Edits {
    Edits {
        basic: preset.basic,
        tone: preset.tone,
        color: preset.color,
        detail: preset.detail,
        effects: preset.effects,
        lens: preset.lens,
        geometry: if params.include_geometry {
            preset.geometry
        } else {
            current.geometry
        },
        masks: if params.include_masks {
            preset.masks
        } else {
            current.masks
        },
        output: if params.include_output {
            preset.output
        } else {
            current.output
        },
        unknown_ops: current.unknown_ops,
    }
}

pub async fn run_apply_preset_item(state: &AppState, job: &JobRecord, asset_id: Uuid) -> ItemOutcome {
    let params: ApplyPresetParams = serde_json::from_value(job.params.clone())
        .map_err(|e| format!("invalid apply preset params: {e}"))?;
    let preset = state
        .edits
        .get_preset(params.preset_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "preset not found".to_string())?;
    let current = state
        .edits
        .get_edits_or_default(asset_id)
        .await
        .map_err(|e| e.to_string())?;
    let merged = merge_preset(current, preset.manifest.to_edits(), &params);
    let manifest = EditManifest::from_edits(&merged);
    let asset = state
        .immich
        .asset(asset_id)
        .await
        .map_err(|e| e.to_string())?;
    let action = format!("Apply preset: {}", preset.name);
    let saved = state
        .edits
        .put(asset_id, manifest, asset.updated_at, asset.checksum, Some(&action))
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "hash": saved.hash,
        "updated_at": saved.updated_at,
        "preset": preset.name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_pipeline::edits::TonemapKind;

    fn params(geometry: bool, output: bool) -> ApplyPresetParams {
        ApplyPresetParams {
            preset_id: Uuid::nil(),
            include_geometry: geometry,
            include_masks: false,
            include_output: output,
        }
    }

    fn preset_edits() -> Edits {
        let mut e = Edits::default();
        e.effects.vignette_amount = 40.0;
        e.geometry.rotate = 2;
        e.output.tonemap = TonemapKind::Agx;
        e
    }

    #[test]
    fn look_groups_always_replaced() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(false, false));
        assert_eq!(merged.effects.vignette_amount, 40.0);
    }

    #[test]
    fn excluded_groups_keep_current() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(false, false));
        assert_eq!(merged.geometry.rotate, 0);
        assert_eq!(merged.output.tonemap, TonemapKind::Default);
    }

    #[test]
    fn included_groups_take_preset() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(true, true));
        assert_eq!(merged.geometry.rotate, 2);
        assert_eq!(merged.output.tonemap, TonemapKind::Agx);
    }
}
