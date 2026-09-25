use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::services::job_runner::{ItemOutcome, JobExecutor, JobItemError};
use crate::services::job_store::{JobItemRecord, JobRecord};
use crate::services::render::RenderIdentity;
use crate::state::AppState;

use super::archive::write_unique;
use super::*;

#[derive(Debug, Deserialize)]
pub struct ExportJobParams {
    #[serde(flatten)]
    pub params: ExportParams,
    #[serde(default)]
    pub album_ids: Vec<Uuid>,
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub stack_with_original: bool,
    #[serde(default)]
    pub stack_primary: StackPrimary,
    #[serde(default, rename = "filename_suffix")]
    legacy_suffix: Option<String>,
    #[serde(default)]
    pub manifest: Option<EditManifest>,
}

impl ExportJobParams {
    fn parse(value: &serde_json::Value) -> Result<Self, JobItemError> {
        let mut parsed: Self = serde_json::from_value(value.clone())
            .map_err(|e| JobItemError::msg(format!("invalid export params: {e}")))?;
        if parsed.params.filename_template.is_none()
            && let Some(suffix) = parsed.legacy_suffix.take().filter(|s| !s.trim().is_empty())
        {
            parsed.params.filename_template = Some(format!("{{name}}{}", suffix.trim()));
        }
        Ok(parsed)
    }
}

fn item_seq(job: &JobRecord, item: &JobItemRecord) -> Seq {
    Seq {
        position: u32::try_from(item.position).unwrap_or(1).max(1),
        total: u32::try_from(job.total).unwrap_or(1).max(1),
    }
}

pub struct BatchExecutor {
    state: AppState,
}

impl BatchExecutor {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl JobExecutor for BatchExecutor {
    fn execute(
        &self,
        job: JobRecord,
        item: JobItemRecord,
    ) -> Pin<Box<dyn Future<Output = ItemOutcome> + Send>> {
        let state = self.state.clone();
        Box::pin(async move {
            let asset_id = item
                .asset_id
                .parse::<AssetKey>()
                .map_err(|_| JobItemError::msg("invalid asset id"))?;
            match job.kind.as_str() {
                EXPORT_JOB_KIND => run_immich_item(&state, &job, &item, asset_id).await,
                DOWNLOAD_ZIP_KIND => run_zip_item(&state, &job, &item, asset_id).await,
                crate::services::apply_preset::APPLY_PRESET_KIND => {
                    crate::services::apply_preset::run_apply_preset_item(&state, &job, asset_id)
                        .await
                }
                crate::services::paste_edits::PASTE_EDITS_KIND => {
                    crate::services::paste_edits::run_paste_edits_item(&state, &job, asset_id).await
                }
                crate::services::reset_edits::RESET_EDITS_KIND => {
                    crate::services::reset_edits::run_reset_edits_item(&state, &job, asset_id).await
                }
                other => Err(JobItemError::msg(format!("unsupported job kind: {other}"))),
            }
        })
    }
}

async fn job_edits(
    state: &AppState,
    owner: Uuid,
    params: &ExportJobParams,
    asset_id: AssetKey,
) -> Result<Edits, JobItemError> {
    match &params.manifest {
        Some(manifest) => Ok(manifest.to_edits()),
        None => Ok(state.edits.get_edits_or_default(owner, asset_id).await?),
    }
}

pub async fn job_immich(
    state: &AppState,
    job: &JobRecord,
) -> Result<crate::immich::ImmichClient, JobItemError> {
    let (cred, kind) = state
        .jobs
        .job_credential(job.id)
        .await?
        .ok_or_else(|| JobItemError::msg("job credential unavailable"))?;
    let cred = cred
        .to_utf8()
        .ok_or_else(|| JobItemError::msg("job credential invalid"))?;
    let cfg = state.instance.get().await?;
    if cfg.server_epoch != job.server_epoch {
        return Err(JobItemError::msg("job belongs to a previous Immich server"));
    }
    let url = cfg
        .immich_url
        .ok_or_else(|| JobItemError::msg("instance not configured"))?;
    let base = url::Url::parse(&url)?;
    let auth = kind.immich_auth(cred);
    crate::immich::ImmichClient::with_auth(
        base,
        auth,
        std::time::Duration::from_secs(state.config.original_timeout_secs),
    )
    .map_err(JobItemError::from)
}

async fn run_immich_item(
    state: &AppState,
    job: &JobRecord,
    item: &JobItemRecord,
    asset_id: AssetKey,
) -> ItemOutcome {
    let immich = job_immich(state, job).await?;
    let params = ExportJobParams::parse(&job.params)?;
    let edits = job_edits(state, job.user_id, &params, asset_id).await?;
    let body = ExportToImmichBody {
        edits: edits.clamped(),
        params: params.params,
        album_ids: params.album_ids,
        tag_ids: params.tag_ids,
        favorite: params.favorite,
        stack_with_original: params.stack_with_original,
        stack_primary: params.stack_primary,
    };
    let idempotency_key = format!("job-{}", job.id);
    let result = export_to_immich(
        state,
        &immich,
        job.user_id,
        ExportImmichRequest {
            asset_id,
            server_epoch: job.server_epoch,
            body: &body,
            idempotency_key: Some(idempotency_key),
            priority: RenderPriority::Background,
            seq: item_seq(job, item),
        },
    )
    .await?;
    Ok(serde_json::to_value(result)?)
}

async fn run_zip_item(
    state: &AppState,
    job: &JobRecord,
    item: &JobItemRecord,
    asset_id: AssetKey,
) -> ItemOutcome {
    let immich = job_immich(state, job).await?;
    let params = ExportJobParams::parse(&job.params)?;
    let edits = job_edits(state, job.user_id, &params, asset_id).await?;
    let template = NameTemplate::parse(params.params.filename_template.as_deref())?;
    let original = immich.asset(asset_id.source()).await?;
    let (bytes, output) = render_export(
        state,
        RenderIdentity {
            owner: job.user_id,
            server_epoch: job.server_epoch,
        },
        &immich,
        asset_id,
        edits.clamped(),
        &params.params,
        RenderPriority::Background,
    )
    .await?;
    let dir = zip_job_dir(state, job.server_epoch, job.user_id, job.id)
        .map_err(|e| JobItemError::msg(format!("export dir: {e}")))?;
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| JobItemError::msg(format!("create export dir: {e}")))?;
    let stem = template.render(&NameContext {
        original: &original.original_file_name,
        date: capture_date(&original),
        seq: item_seq(job, item),
    });
    let filename = write_unique(&dir, &stem, output.extension(), &bytes)
        .await
        .map_err(|e| JobItemError::msg(format!("write export file: {e}")))?;
    Ok(serde_json::json!({ "filename": filename, "bytes": bytes.len() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template_of(params: serde_json::Value) -> Option<String> {
        ExportJobParams::parse(&params)
            .unwrap()
            .params
            .filename_template
    }

    #[test]
    fn a_legacy_suffix_becomes_a_name_template() {
        assert_eq!(
            template_of(serde_json::json!({ "filename_suffix": "_warm" })).as_deref(),
            Some("{name}_warm")
        );
    }

    #[test]
    fn a_template_wins_over_a_legacy_suffix() {
        let params =
            serde_json::json!({ "filename_template": "{seq}", "filename_suffix": "_warm" });
        assert_eq!(template_of(params).as_deref(), Some("{seq}"));
    }

    #[test]
    fn a_blank_legacy_suffix_keeps_the_default() {
        assert_eq!(
            template_of(serde_json::json!({ "filename_suffix": " " })),
            None
        );
    }
}
