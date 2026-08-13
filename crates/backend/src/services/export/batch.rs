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

use super::archive::{sanitize_filename, write_unique};
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
    #[serde(default = "default_suffix")]
    pub filename_suffix: String,
    #[serde(default)]
    pub manifest: Option<EditManifest>,
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
                EXPORT_JOB_KIND => run_immich_item(&state, &job, asset_id).await,
                DOWNLOAD_ZIP_KIND => run_zip_item(&state, &job, asset_id).await,
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

fn parse_job_params(job: &JobRecord) -> Result<ExportJobParams, JobItemError> {
    serde_json::from_value(job.params.clone())
        .map_err(|e| JobItemError::msg(format!("invalid export params: {e}")))
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
    let auth = match kind {
        crate::services::auth_store::AuthKind::Password => {
            crate::immich::client::ImmichAuth::Bearer(cred)
        }
        crate::services::auth_store::AuthKind::ApiKey => {
            crate::immich::client::ImmichAuth::ApiKey(cred)
        }
    };
    crate::immich::ImmichClient::with_auth(
        base,
        auth,
        std::time::Duration::from_secs(state.config.original_timeout_secs),
    )
    .map_err(JobItemError::from)
}

async fn run_immich_item(state: &AppState, job: &JobRecord, asset_id: AssetKey) -> ItemOutcome {
    let immich = job_immich(state, job).await?;
    let params = parse_job_params(job)?;
    let edits = job_edits(state, job.user_id, &params, asset_id).await?;
    let body = ExportToImmichBody {
        edits: edits.clamped(),
        params: params.params,
        album_ids: params.album_ids,
        tag_ids: params.tag_ids,
        favorite: params.favorite,
        stack_with_original: params.stack_with_original,
        stack_primary: params.stack_primary,
        filename_suffix: params.filename_suffix,
    };
    let device_asset_id = format!("immich-edit-job-{}-{}", job.id, asset_id);
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
            device_asset_id: Some(device_asset_id),
        },
    )
    .await?;
    Ok(serde_json::to_value(result)?)
}

async fn run_zip_item(state: &AppState, job: &JobRecord, asset_id: AssetKey) -> ItemOutcome {
    let immich = job_immich(state, job).await?;
    let params = parse_job_params(job)?;
    let edits = job_edits(state, job.user_id, &params, asset_id).await?;
    let suffix = validate_suffix(&params.filename_suffix)?;
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
    )
    .await?;
    let dir = zip_job_dir(state, job.server_epoch, job.user_id, job.id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| JobItemError::msg(format!("create export dir: {e}")))?;
    let base = sanitize_filename(&original.original_file_name);
    let filename = write_unique(&dir, &base, &suffix, output.extension(), &bytes)
        .await
        .map_err(|e| JobItemError::msg(format!("write export file: {e}")))?;
    Ok(serde_json::json!({ "filename": filename, "bytes": bytes.len() }))
}
