use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::client::UploadRequest;
use crate::services::edits_store::{ExportJobKey, ExportJobRecord, ExportJobStatus};
use crate::services::render::RenderIdentity;
use crate::services::render_queue::RenderPriority;
use crate::state::AppState;
use crate::telemetry::ErrorChain;

use super::post_upload::run_post_upload;
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
}

#[derive(Debug, Serialize)]
pub struct ExportToImmichResult {
    pub asset_id: Uuid,
    pub filename: String,
    pub status: String,
    pub warnings: Vec<String>,
}

pub struct ExportImmichRequest<'a> {
    pub asset_id: AssetKey,
    pub server_epoch: i64,
    pub body: &'a ExportToImmichBody,
    pub idempotency_key: Option<String>,
    pub priority: RenderPriority,
    pub seq: Seq,
}

pub async fn export_to_immich(
    state: &AppState,
    immich: &ImmichClient,
    owner: Uuid,
    req: ExportImmichRequest<'_>,
) -> Result<ExportToImmichResult, AppError> {
    let id = req.asset_id;
    let body = req.body;
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
    let mut reserved = false;

    if let (Some(job), Some(hash)) = (job, request_hash.as_deref()) {
        reserved = state.edits.reserve_export_job(job, hash).await?;
        if !reserved && let Some(existing) = state.edits.get_export_job(job).await? {
            if existing.request_hash != hash {
                return Err(AppError::BadRequest(
                    "idempotency key reused with different request".into(),
                ));
            }
            return match existing.status {
                ExportJobStatus::Pending => {
                    Err(AppError::Conflict("export already in progress".into()))
                }
                ExportJobStatus::Uploaded => {
                    resume_export_job(state, immich, identity, job, body, existing).await
                }
                ExportJobStatus::Completed => Ok(record_to_result(&existing)),
            };
        }
    }

    let result: Result<ExportToImmichResult, AppError> = async {
        let template = NameTemplate::parse(body.params.filename_template.as_deref())?;
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

        let warnings: Vec<String> = rendered
            .warnings
            .into_iter()
            .chain(run_post_upload(state, identity, immich, &original, body, new_id, &status).await)
            .collect();

        if let Some(job) = job {
            state.edits.complete_export_job(job, &warnings).await?;
        }

        Ok(ExportToImmichResult {
            asset_id: new_id,
            filename,
            status,
            warnings,
        })
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

pub fn hash_request(asset_id: AssetKey, body: &ExportToImmichBody) -> String {
    let mut album_ids = body.album_ids.clone();
    album_ids.sort();
    let mut tag_ids = body.tag_ids.clone();
    tag_ids.sort();
    let canonical = serde_json::json!({
        "asset_id": asset_id.to_string(),
        "edits": body.edits.clamped(),
        "format": format!("{:?}", body.params.format),
        "quality": body.params.quality,
        "metadata": format!("{:?}", body.params.metadata()),
        "bit_depth": format!("{:?}", body.params.bit_depth),
        "png_compression": format!("{:?}", body.params.png_compression),
        "tiff_compression": format!("{:?}", body.params.tiff_compression),
        "lossless": body.params.lossless,
        "color_space": format!("{:?}", body.params.color_space),
        "album_ids": album_ids,
        "tag_ids": tag_ids,
        "favorite": body.favorite,
        "stack_with_original": body.stack_with_original,
        "stack_primary": format!("{:?}", body.stack_primary),
        "filename_template": body.params.filename_template,
        "resize_mode": body.params.resize_mode.map(|m| format!("{m:?}")),
        "resize_width": body.params.resize_width,
        "resize_height": body.params.resize_height,
        "resize_megapixels": body.params.resize_megapixels,
        "resize_percent": body.params.resize_percent,
        "resize_enlarge": body.params.resize_enlarge,
        "output_sharpen_media": body.params.output_sharpen_media.map(|m| format!("{m:?}")),
        "output_sharpen_amount": format!("{:?}", body.params.output_sharpen_amount),
        "output_sharpen_ppi": body.params.output_sharpen_ppi,
        "watermark_id": body.params.watermark_id,
        "watermark_size": body.params.watermark_size,
        "watermark_opacity": body.params.watermark_opacity,
        "watermark_anchor": format!("{:?}", body.params.watermark_anchor),
        "watermark_inset": body.params.watermark_inset,
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

fn record_to_result(rec: &ExportJobRecord) -> ExportToImmichResult {
    ExportToImmichResult {
        asset_id: rec.immich_asset_id.unwrap_or_default(),
        filename: rec.filename.clone().unwrap_or_default(),
        status: rec.upload_status.clone().unwrap_or_default(),
        warnings: rec.warnings.clone(),
    }
}

async fn resume_export_job(
    state: &AppState,
    immich: &ImmichClient,
    identity: RenderIdentity,
    job: ExportJobKey<'_>,
    body: &ExportToImmichBody,
    existing: ExportJobRecord,
) -> Result<ExportToImmichResult, AppError> {
    let Some(new_id) = existing.immich_asset_id else {
        tracing::error!(asset = %job.asset_id, "export job record has no immich asset id");
        return Err(AppError::Internal);
    };
    let original = immich.asset(job.asset_id.source()).await?;
    let upload_status = existing.upload_status.clone().unwrap_or_default();
    let warnings: Vec<String> = existing
        .warnings
        .into_iter()
        .chain(
            run_post_upload(
                state,
                identity,
                immich,
                &original,
                body,
                new_id,
                &upload_status,
            )
            .await,
        )
        .collect();
    state.edits.complete_export_job(job, &warnings).await?;
    Ok(ExportToImmichResult {
        asset_id: new_id,
        filename: existing.filename.unwrap_or_default(),
        status: upload_status,
        warnings,
    })
}
