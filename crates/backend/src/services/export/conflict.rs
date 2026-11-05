use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::edits_store::{ExportJobCompletion, ExportJobKey, ExportOutputKey};
use crate::services::keyed_lock::KeyedGuard;
use crate::state::AppState;

use super::ExportToImmichResult;

const SKIPPED: &str = "skipped";
const RECORDED_STATUSES: [&str; 2] = ["created", "duplicate"];

pub(super) type OutputTurn = KeyedGuard<(Uuid, String, String)>;

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    #[default]
    KeepBoth,
    Skip,
    Replace,
}

impl ConflictPolicy {
    pub(super) fn hashed(self) -> Option<&'static str> {
        match self {
            Self::KeepBoth => None,
            Self::Skip => Some("skip"),
            Self::Replace => Some("replace"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Previous {
    Live,
    Trashed,
    Gone,
}

#[derive(Debug, PartialEq, Eq)]
enum Plan {
    Export,
    Skip,
    Replace,
}

fn plan(policy: ConflictPolicy, previous: Option<Previous>) -> Plan {
    match (policy, previous) {
        (ConflictPolicy::Skip, Some(Previous::Live)) => Plan::Skip,
        (ConflictPolicy::Replace, Some(Previous::Live)) => Plan::Replace,
        _ => Plan::Export,
    }
}

pub(super) enum Conflict {
    None,
    Skip(Uuid, String),
    Replace(Uuid),
}

pub(super) struct Resolved {
    pub previous: Option<Uuid>,
    pub conflict: Conflict,
}

pub(super) async fn hold(state: &AppState, key: ExportOutputKey<'_>) -> OutputTurn {
    let turn = (key.owner, key.asset_id.to_string(), key.preset.to_string());
    state.export_output_locks.lock(turn).await
}

pub(super) async fn resolve(
    state: &AppState,
    immich: &ImmichClient,
    policy: ConflictPolicy,
    key: ExportOutputKey<'_>,
) -> Result<Resolved, AppError> {
    let previous = state.edits.get_export_output(key).await?;
    let conflict = match previous {
        Some(id) if policy != ConflictPolicy::KeepBoth => conflict_with(immich, policy, id).await?,
        _ => Conflict::None,
    };
    Ok(Resolved { previous, conflict })
}

async fn conflict_with(
    immich: &ImmichClient,
    policy: ConflictPolicy,
    id: Uuid,
) -> Result<Conflict, AppError> {
    let (previous, filename) = match immich.asset(id).await {
        Ok(detail) if detail.is_trashed => (Previous::Trashed, None),
        Ok(detail) => (Previous::Live, Some(detail.original_file_name)),
        Err(err) if is_missing(&err) => (Previous::Gone, None),
        Err(err) => return Err(err.into()),
    };
    Ok(match plan(policy, Some(previous)) {
        Plan::Export => Conflict::None,
        Plan::Skip => Conflict::Skip(id, filename.unwrap_or_default()),
        Plan::Replace => Conflict::Replace(id),
    })
}

pub(super) fn is_missing(err: &ImmichError) -> bool {
    matches!(
        err,
        ImmichError::NotFound | ImmichError::Rejected { status: 400, .. }
    )
}

pub(super) fn skipped(previous: Uuid, filename: String) -> ExportToImmichResult {
    ExportToImmichResult {
        asset_id: previous,
        filename,
        status: SKIPPED.into(),
        warnings: Vec::new(),
        original_trashed: false,
        replaced_asset_id: None,
    }
}

pub(super) fn resume_target(
    replace: Option<Uuid>,
    current: Option<Uuid>,
    previous: Option<Uuid>,
    upload: Uuid,
) -> Option<Uuid> {
    replace.filter(|_| current == previous || current == Some(upload))
}

fn is_recorded(key: ExportOutputKey<'_>, result: &ExportToImmichResult) -> bool {
    let uploaded = RECORDED_STATUSES
        .iter()
        .any(|status| result.status.eq_ignore_ascii_case(status));
    uploaded && result.asset_id != key.asset_id.source()
}

pub(super) async fn record(
    state: &AppState,
    job: Option<ExportJobKey<'_>>,
    key: ExportOutputKey<'_>,
    previous: Option<Uuid>,
    result: &ExportToImmichResult,
) -> Result<(), AppError> {
    let output = is_recorded(key, result).then(|| key.output(result.asset_id, previous));
    let written = match (job, output) {
        (Some(job), output) => {
            let completion = ExportJobCompletion {
                warnings: &result.warnings,
                original_trashed: result.original_trashed,
                replaced_asset_id: result.replaced_asset_id,
                output,
            };
            state.edits.complete_export_job(job, completion).await?
        }
        (None, Some(output)) => state.edits.put_export_output(output).await?,
        (None, None) => false,
    };
    if output.is_some() && !written {
        tracing::debug!(
            asset = %key.asset_id,
            preset = key.preset,
            upload = %result.asset_id,
            expected = ?previous,
            "export output already moved on; kept the newer record"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::asset_key::AssetKey;
    use crate::services::export::{ExportJobParams, ExportToImmichBody};

    const POLICIES: [ConflictPolicy; 3] = [
        ConflictPolicy::KeepBoth,
        ConflictPolicy::Skip,
        ConflictPolicy::Replace,
    ];

    fn policy_of(value: serde_json::Value) -> Result<ConflictPolicy, serde_json::Error> {
        serde_json::from_value::<ExportToImmichBody>(value).map(|body| body.on_conflict)
    }

    fn plans(previous: Option<Previous>) -> Vec<Plan> {
        POLICIES
            .into_iter()
            .map(|policy| plan(policy, previous))
            .collect()
    }

    #[test]
    fn the_policy_defaults_to_keep_both_and_rejects_unknown_values() {
        assert_eq!(policy_of(json!({})).unwrap(), ConflictPolicy::KeepBoth);
        for (name, policy) in ["keep_both", "skip", "replace"].into_iter().zip(POLICIES) {
            assert_eq!(policy_of(json!({ "on_conflict": name })).unwrap(), policy);
        }
        for bad in [json!("overwrite"), json!("Skip"), json!(1)] {
            assert!(policy_of(json!({ "on_conflict": bad })).is_err(), "{bad}");
        }
        assert!(ExportJobParams::parse(&json!({ "on_conflict": "overwrite" })).is_err());
        assert_eq!(
            ExportJobParams::parse(&json!({ "on_conflict": "replace" }))
                .unwrap()
                .on_conflict,
            ConflictPolicy::Replace
        );
    }

    #[test]
    fn only_a_live_previous_export_conflicts() {
        for previous in [None, Some(Previous::Gone), Some(Previous::Trashed)] {
            assert_eq!(
                plans(previous),
                [Plan::Export, Plan::Export, Plan::Export],
                "{previous:?}"
            );
        }
    }

    #[test]
    fn a_live_previous_export_follows_the_policy() {
        assert_eq!(
            plans(Some(Previous::Live)),
            [Plan::Export, Plan::Skip, Plan::Replace]
        );
    }

    #[test]
    fn hashing_names_only_a_policy_other_than_keep_both() {
        let hashed: Vec<Option<&str>> = POLICIES.into_iter().map(|p| p.hashed()).collect();

        assert_eq!(hashed, [None, Some("skip"), Some("replace")]);
    }

    #[test]
    fn a_resume_replaces_only_while_the_output_has_not_moved_on() {
        let [replace, upload, newer] = [(); 3].map(|_| Uuid::new_v4());
        let previous = Some(replace);

        for (target, current, want) in [
            (Some(replace), previous, Some(replace)),
            (Some(replace), Some(upload), Some(replace)),
            (Some(replace), Some(newer), None),
            (Some(replace), None, None),
            (None, previous, None),
        ] {
            assert_eq!(
                resume_target(target, current, previous, upload),
                want,
                "{current:?}"
            );
        }
    }

    fn result(asset_id: Uuid, status: &str) -> ExportToImmichResult {
        ExportToImmichResult {
            asset_id,
            filename: "a.jpg".into(),
            status: status.into(),
            warnings: Vec::new(),
            original_trashed: false,
            replaced_asset_id: None,
        }
    }

    #[test]
    fn only_an_upload_other_than_the_original_is_recorded() {
        let source = Uuid::new_v4();
        let key = ExportOutputKey {
            owner: Uuid::nil(),
            asset_id: AssetKey::master(source),
            preset: "",
        };
        let upload = Uuid::new_v4();

        for (id, status, want) in [
            (upload, "created", true),
            (upload, "DUPLICATE", true),
            (upload, "replaced", false),
            (upload, SKIPPED, false),
            (source, "duplicate", false),
        ] {
            assert_eq!(is_recorded(key, &result(id, status)), want, "{status}");
        }
    }
}
