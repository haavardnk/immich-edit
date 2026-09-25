use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::{Edits, LOOK_AMOUNT_FULL};
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::services::edit_merge::{MergeSections, merge_edits};
use crate::services::job_runner::{ItemOutcome, JobItemError};
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
    #[serde(default = "full_amount")]
    pub amount: f64,
}

fn full_amount() -> f64 {
    LOOK_AMOUNT_FULL
}

pub fn merge_preset(current: Edits, preset: Edits, params: &ApplyPresetParams) -> Edits {
    let sections = MergeSections {
        geometry: params.include_geometry,
        masks: params.include_masks,
        ..MergeSections::look_only()
    };
    merge_edits(current, preset.with_look_amount(params.amount), sections)
}

fn preset_action(name: &str, amount: f64) -> String {
    if amount == LOOK_AMOUNT_FULL {
        format!("Apply preset: {name}")
    } else {
        format!("Apply preset: {name} ({amount}%)")
    }
}

pub async fn run_apply_preset_item(
    state: &AppState,
    job: &JobRecord,
    asset_id: AssetKey,
) -> ItemOutcome {
    let params: ApplyPresetParams = serde_json::from_value(job.params.clone())
        .map_err(|e| JobItemError::msg(format!("invalid apply preset params: {e}")))?;
    let preset = state
        .edits
        .get_preset(job.user_id, params.preset_id)
        .await?
        .ok_or_else(|| JobItemError::msg("preset not found"))?;
    let current = state
        .edits
        .get_edits_or_default(job.user_id, asset_id)
        .await?;
    let merged = merge_preset(current, preset.manifest.to_edits(), &params);
    let manifest = EditManifest::from_edits(&merged);
    let immich = crate::services::export::job_immich(state, job).await?;
    let asset = immich.asset(asset_id.source()).await?;
    let action = preset_action(&preset.name, params.amount);
    let saved = state
        .edits
        .put(
            job.user_id,
            asset_id,
            manifest,
            asset.updated_at,
            asset.checksum,
            Some(&action),
        )
        .await?;
    Ok(serde_json::json!({
        "hash": saved.hash,
        "updated_at": saved.updated_at,
        "preset": preset.name,
        "amount": params.amount,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(geometry: bool) -> ApplyPresetParams {
        ApplyPresetParams {
            preset_id: Uuid::nil(),
            include_geometry: geometry,
            include_masks: false,
            amount: LOOK_AMOUNT_FULL,
        }
    }

    fn preset_edits() -> Edits {
        let mut e = Edits::default();
        e.effects.vignette_amount = 40.0;
        e.geometry.rotate = 2;
        e
    }

    #[test]
    fn look_groups_always_replaced() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(false));
        assert_eq!(merged.effects.vignette_amount, 40.0);
    }

    #[test]
    fn excluded_groups_keep_current() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(false));
        assert_eq!(merged.geometry.rotate, 0);
    }

    #[test]
    fn included_groups_take_preset() {
        let merged = merge_preset(Edits::default(), preset_edits(), &params(true));
        assert_eq!(merged.geometry.rotate, 2);
    }

    #[test]
    fn amount_scales_the_look_but_not_geometry() {
        let half = ApplyPresetParams {
            amount: 50.0,
            ..params(true)
        };
        let merged = merge_preset(Edits::default(), preset_edits(), &half);
        assert_eq!(merged.effects.vignette_amount, 20.0);
        assert_eq!(merged.geometry.rotate, 2);
    }

    #[test]
    fn amount_defaults_to_full() {
        let parsed: ApplyPresetParams =
            serde_json::from_value(serde_json::json!({ "preset_id": Uuid::nil() })).unwrap();
        assert_eq!(parsed.amount, LOOK_AMOUNT_FULL);
    }

    #[test]
    fn action_names_a_partial_amount() {
        assert_eq!(preset_action("Warm", 100.0), "Apply preset: Warm");
        assert_eq!(preset_action("Warm", 50.0), "Apply preset: Warm (50%)");
    }
}
