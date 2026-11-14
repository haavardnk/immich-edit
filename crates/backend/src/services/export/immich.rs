use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::client::UploadRequest;
use crate::services::edits_store::{ExportJobBaseline, ExportJobKey, ExportOutputKey};
use crate::services::render::RenderIdentity;
use crate::services::render_queue::RenderPriority;
use crate::services::trash;
use crate::state::AppState;
use crate::telemetry::ErrorChain;

use super::conflict::{self, Conflict};
use super::naming::preset_key;
use super::post_upload::{Uploaded, finish_upload};
use super::replay::{ExportContext, settle_existing};
use super::*;

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StackPrimary {
    #[default]
    Edited,
    Original,
}

#[derive(Debug, Deserialize)]
pub struct ExportToImmichBody {
    #[serde(default)]
    pub edits: Edits,
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
    #[serde(default)]
    pub replace_original: bool,
    #[serde(default)]
    pub on_conflict: ConflictPolicy,
}

impl ExportToImmichBody {
    pub fn validate(&self) -> Result<(), AppError> {
        replace_fits(self.replace_original, self.stack_with_original)
    }
}

pub(super) fn replace_fits(
    replace_original: bool,
    stack_with_original: bool,
) -> Result<(), AppError> {
    if replace_original && stack_with_original {
        return Err(AppError::BadRequest(
            "replace original and stack with original cannot be combined".into(),
        ));
    }
    Ok(())
}

pub(super) fn trashes(replace_original: bool, on_conflict: ConflictPolicy) -> bool {
    replace_original || on_conflict == ConflictPolicy::Replace
}

#[derive(Debug, Serialize)]
pub struct ExportToImmichResult {
    pub asset_id: Uuid,
    pub filename: String,
    pub status: String,
    pub warnings: Vec<String>,
    pub original_trashed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_asset_id: Option<Uuid>,
}

pub struct ExportImmichRequest<'a> {
    pub asset_id: AssetKey,
    pub server_epoch: i64,
    pub body: &'a ExportToImmichBody,
    pub idempotency_key: Option<String>,
    pub priority: RenderPriority,
    pub seq: Seq,
    pub preset: Option<&'a str>,
    pub variant: i64,
}

pub async fn export_to_immich(
    state: &AppState,
    immich: &ImmichClient,
    owner: Uuid,
    req: ExportImmichRequest<'_>,
) -> Result<ExportToImmichResult, AppError> {
    let id = req.asset_id;
    let body = req.body;
    body.validate()?;
    let identity = RenderIdentity {
        owner,
        server_epoch: req.server_epoch,
    };
    let job = req.idempotency_key.as_deref().map(|key| ExportJobKey {
        owner,
        asset_id: id,
        key,
    });
    let request_hash = job.map(|_| hash_request(id, body));
    let preset = preset_key(req.preset.unwrap_or_default());
    let output = ExportOutputKey {
        owner,
        asset_id: id,
        preset: &preset,
    };
    let export = ExportContext {
        identity,
        body,
        output,
        variant: req.variant,
    };

    let _turn = conflict::hold(state, output).await;
    if let (Some(job), Some(hash)) = (job, request_hash.as_deref())
        && let Some(existing) = state.edits.get_export_job(job).await?
    {
        return settle_existing(state, immich, export, job, hash, existing).await;
    }
    let template = NameTemplate::parse(body.params.filename_template.as_deref())?;
    if trashes(body.replace_original, body.on_conflict) {
        trash::ensure_enabled(immich).await?;
    }
    let resolved = conflict::resolve(state, immich, body.on_conflict, output).await?;
    let replace = match resolved.conflict {
        Conflict::Skip(previous, filename) => return Ok(conflict::skipped(previous, filename)),
        Conflict::Replace(previous) => Some(previous),
        Conflict::None => None,
    };

    let mut reserved = false;
    if let (Some(job), Some(hash)) = (job, request_hash.as_deref()) {
        let baseline = ExportJobBaseline {
            previous_asset_id: resolved.previous,
            replace_asset_id: replace,
        };
        reserved = state.edits.reserve_export_job(job, hash, baseline).await?;
        if !reserved && let Some(existing) = state.edits.get_export_job(job).await? {
            return settle_existing(state, immich, export, job, hash, existing).await;
        }
    }

    let result: Result<ExportToImmichResult, AppError> = async {
        let original = immich.asset(id.source()).await?;
        let existing_names = collect_existing_filenames(immich, &original).await;

        let rendered = render_export(
            state,
            identity,
            immich,
            id,
            body.edits.clamped(),
            &body.params,
            req.priority,
        )
        .await?;
        let stem = template.render(&NameContext {
            original: &original.original_file_name,
            date: capture_date(&original),
            seq: req.seq,
            preset: req.preset,
        });
        let filename = resolve_filename(&stem, rendered.output.extension(), &existing_names);
        let now = Utc::now().to_rfc3339();
        let created_at = original.file_created_at.as_deref().unwrap_or(&now);
        let upload = immich
            .upload_asset(UploadRequest {
                filename: &filename,
                content_type: rendered.output.content_type(),
                bytes: rendered.bytes,
                is_favorite: body.favorite,
                created_at,
                modified_at: &now,
            })
            .await?;

        let new_id = upload.id;
        let status = upload.status.clone();

        if let (Some(job), Some(hash)) = (job, request_hash.as_deref()) {
            state
                .edits
                .put_export_job_uploaded(job, hash, new_id, &filename, &status, &rendered.warnings)
                .await?;
        }

        let finished = finish_upload(
            state,
            identity,
            immich,
            Uploaded {
                original: &original,
                body,
                id: new_id,
                status: &status,
                variant: req.variant,
                output,
                previous: replace,
            },
        )
        .await;
        let warnings: Vec<String> = rendered
            .warnings
            .into_iter()
            .chain(finished.warnings)
            .collect();

        let result = ExportToImmichResult {
            asset_id: new_id,
            filename,
            status,
            warnings,
            original_trashed: finished.original_trashed,
            replaced_asset_id: finished.replaced_asset_id,
        };
        conflict::record(state, job, output, resolved.previous, &result).await?;
        Ok(result)
    }
    .await;

    if result.is_err()
        && reserved
        && let Some(job) = job
        && let Err(error) = state.edits.delete_pending_export_job(job).await
    {
        tracing::warn!(error = %ErrorChain(&error), "release pending export job");
    }
    result
}
