use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::services::edits_store::{
    ExportJobKey, ExportJobRecord, ExportJobStatus, ExportOutputKey,
};
use crate::services::render::RenderIdentity;
use crate::state::AppState;

use super::conflict;
use super::post_upload::{Uploaded, finish_upload};
use super::{ExportToImmichBody, ExportToImmichResult};

#[derive(Clone, Copy)]
pub(super) struct ExportContext<'a> {
    pub identity: RenderIdentity,
    pub body: &'a ExportToImmichBody,
    pub output: ExportOutputKey<'a>,
    pub variant: i64,
}

pub(super) async fn settle_existing(
    state: &AppState,
    immich: &ImmichClient,
    export: ExportContext<'_>,
    job: ExportJobKey<'_>,
    request_hash: &str,
    existing: ExportJobRecord,
) -> Result<ExportToImmichResult, AppError> {
    if existing.request_hash != request_hash {
        return Err(AppError::BadRequest(
            "idempotency key reused with different request".into(),
        ));
    }
    match existing.status {
        ExportJobStatus::Pending => Err(AppError::Conflict("export already in progress".into())),
        ExportJobStatus::Uploaded => resume_export_job(state, immich, export, job, existing).await,
        ExportJobStatus::Completed => Ok(record_to_result(&existing)),
    }
}

fn record_to_result(rec: &ExportJobRecord) -> ExportToImmichResult {
    ExportToImmichResult {
        asset_id: rec.immich_asset_id.unwrap_or_default(),
        filename: rec.filename.clone().unwrap_or_default(),
        status: rec.upload_status.clone().unwrap_or_default(),
        warnings: rec.warnings.clone(),
        original_trashed: rec.original_trashed,
        replaced_asset_id: rec.replaced_asset_id,
    }
}

async fn resume_export_job(
    state: &AppState,
    immich: &ImmichClient,
    export: ExportContext<'_>,
    job: ExportJobKey<'_>,
    existing: ExportJobRecord,
) -> Result<ExportToImmichResult, AppError> {
    let Some(new_id) = existing.immich_asset_id else {
        tracing::error!(asset = %job.asset_id, "export job record has no immich asset id");
        return Err(AppError::Internal);
    };
    let original = immich.asset(job.asset_id.source()).await?;
    let previous = existing.previous_asset_id;
    let current = state.edits.get_export_output(export.output).await?;
    let replace = conflict::resume_target(existing.replace_asset_id, current, previous, new_id);
    let upload_status = existing.upload_status.clone().unwrap_or_default();
    let finished = finish_upload(
        state,
        export.identity,
        immich,
        Uploaded {
            original: &original,
            body: export.body,
            id: new_id,
            status: &upload_status,
            variant: export.variant,
            output: export.output,
            previous: replace,
        },
    )
    .await;
    let warnings: Vec<String> = existing
        .warnings
        .into_iter()
        .chain(finished.warnings)
        .collect();
    let result = ExportToImmichResult {
        asset_id: new_id,
        filename: existing.filename.unwrap_or_default(),
        status: upload_status,
        warnings,
        original_trashed: finished.original_trashed,
        replaced_asset_id: finished.replaced_asset_id,
    };
    conflict::record(state, Some(job), export.output, previous, &result).await?;
    Ok(result)
}
