use bytes::Bytes;
use chrono::Utc;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::OutputFormat;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::services::edits_store::{ExportJobRecord, ExportJobStatus};
use crate::services::render::RenderIdentity;
use crate::state::AppState;

pub const EXPORT_MAX_EDGE: u32 = 65535;
pub const DEFAULT_QUALITY: u8 = 90;
pub const EXPORT_JOB_KIND: &str = "export_immich";
pub const DOWNLOAD_ZIP_KIND: &str = "download_zip";

mod archive;
mod batch;
mod naming;
mod params;

pub use archive::*;
pub use batch::*;
pub use naming::*;
pub use params::*;

#[derive(Debug, Deserialize)]
pub struct ExportBody {
    #[serde(default)]
    pub edits: Edits,
    #[serde(flatten)]
    pub params: ExportParams,
}

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
    #[serde(default = "default_suffix")]
    pub filename_suffix: String,
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
    pub device_asset_id: Option<String>,
}

pub async fn render_export(
    state: &AppState,
    identity: RenderIdentity,
    immich: &crate::immich::ImmichClient,
    id: AssetKey,
    edits: Edits,
    params: &ExportParams,
) -> Result<(Bytes, OutputFormat), AppError> {
    let frame = state
        .render
        .frame(identity, immich, id.source())
        .await
        .map_err(AppError::from)?;
    let output = params.output_format();
    let opts = raw_pipeline::frame::RenderOptions {
        max_edge: EXPORT_MAX_EDGE,
        quality: true,
        output,
        output_color_space: params.output_color_space(),
        ..Default::default()
    };
    let rendered = state
        .render
        .render(identity, immich.clone(), id.source(), edits, opts, None)
        .await
        .map_err(AppError::from)?;

    let mut bytes = rendered.bytes;
    if params.include_exif
        && let Some(exif) = frame.exif.as_ref()
        && let Err(e) = raw_pipeline::exif::inject(&mut bytes, exif, output.exif_file_extension())
    {
        tracing::warn!(error = %e, "exif inject failed");
    }
    Ok((Bytes::from(bytes), output))
}

pub async fn export_to_immich(
    state: &AppState,
    immich: &crate::immich::ImmichClient,
    owner: Uuid,
    req: ExportImmichRequest<'_>,
) -> Result<ExportToImmichResult, AppError> {
    let id = req.asset_id;
    let body = req.body;
    let idem_key = req.idempotency_key;
    let request_hash = idem_key.as_ref().map(|_| hash_request(id, body));
    let mut reserved = false;

    if let (Some(key), Some(hash)) = (idem_key.as_deref(), request_hash.as_deref()) {
        reserved = state.edits.reserve_export_job(owner, id, key, hash).await?;
        if !reserved && let Some(existing) = state.edits.get_export_job(owner, id, key).await? {
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
                    resume_export_job(
                        state,
                        immich,
                        owner,
                        req.server_epoch,
                        id,
                        key,
                        body,
                        existing,
                    )
                    .await
                }
                ExportJobStatus::Completed => Ok(record_to_result(&existing)),
            };
        }
    }

    let result: Result<ExportToImmichResult, AppError> = async {
        let suffix = validate_suffix(&body.filename_suffix)?;
        let original = immich.asset(id.source()).await?;
        let existing_names = collect_existing_filenames(immich, &original).await;

        let (bytes, output) = render_export(
            state,
            RenderIdentity {
                owner,
                server_epoch: req.server_epoch,
            },
            immich,
            id,
            body.edits.clamped(),
            &body.params,
        )
        .await?;
        let filename = resolve_filename(
            &original.original_file_name,
            &suffix,
            output.extension(),
            &existing_names,
        );
        let device_asset_id = req.device_asset_id.unwrap_or_else(|| {
            request_hash
                .as_deref()
                .map(|hash| format!("immich-edit-{id}-{hash}"))
                .unwrap_or_else(|| format!("immich-edit-{filename}"))
        });
        let now = Utc::now().to_rfc3339();
        let upload = immich
            .upload_asset(crate::immich::client::UploadRequest {
                filename: &filename,
                content_type: output.content_type(),
                bytes,
                is_favorite: body.favorite,
                created_at: &now,
                modified_at: &now,
                device_asset_id: &device_asset_id,
            })
            .await?;

        let new_id = upload.id;
        let status = upload.status.clone();

        if let (Some(key), Some(hash)) = (idem_key.as_deref(), request_hash.as_deref()) {
            state
                .edits
                .put_export_job_uploaded(owner, id, key, hash, new_id, &filename, &status)
                .await?;
        }

        let warnings = run_post_upload(
            state,
            owner,
            req.server_epoch,
            immich,
            &original,
            body,
            new_id,
            &status,
        )
        .await;

        if let Some(key) = idem_key.as_deref() {
            state
                .edits
                .complete_export_job(owner, id, key, &warnings)
                .await?;
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
        && let Some(key) = idem_key.as_deref()
    {
        let _ = state.edits.delete_pending_export_job(owner, id, key).await;
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
        "include_exif": body.params.include_exif,
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
        "filename_suffix": body.filename_suffix,
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

#[allow(clippy::too_many_arguments)]
async fn resume_export_job(
    state: &AppState,
    immich: &crate::immich::ImmichClient,
    owner: Uuid,
    server_epoch: i64,
    asset_id: AssetKey,
    key: &str,
    body: &ExportToImmichBody,
    existing: ExportJobRecord,
) -> Result<ExportToImmichResult, AppError> {
    let Some(new_id) = existing.immich_asset_id else {
        return Err(AppError::Internal);
    };
    let original = immich.asset(asset_id.source()).await?;
    let upload_status = existing.upload_status.clone().unwrap_or_default();
    let warnings = run_post_upload(
        state,
        owner,
        server_epoch,
        immich,
        &original,
        body,
        new_id,
        &upload_status,
    )
    .await;
    state
        .edits
        .complete_export_job(owner, asset_id, key, &warnings)
        .await?;
    Ok(ExportToImmichResult {
        asset_id: new_id,
        filename: existing.filename.unwrap_or_default(),
        status: upload_status,
        warnings,
    })
}

#[allow(clippy::too_many_arguments)]
async fn run_post_upload(
    state: &AppState,
    owner: Uuid,
    server_epoch: i64,
    immich: &crate::immich::ImmichClient,
    original: &AssetDetail,
    body: &ExportToImmichBody,
    new_id: Uuid,
    upload_status: &str,
) -> Vec<String> {
    let mut warnings: Vec<String> = Vec::new();
    let is_duplicate = upload_status.eq_ignore_ascii_case("duplicate");

    if body.favorite
        && is_duplicate
        && let Err(e) = immich
            .update_asset(new_id, &serde_json::json!({ "isFavorite": true }))
            .await
    {
        warnings.push(format!("Favorite failed: {}", e.short()));
    }

    for album_id in &body.album_ids {
        match immich.add_assets_to_album(*album_id, &[new_id]).await {
            Ok(items) => {
                for item in items {
                    if !item.success {
                        warnings.push(format!(
                            "Album {album_id} failed: {}",
                            item.error.unwrap_or_else(|| "unknown".into())
                        ));
                    }
                }
            }
            Err(e) => warnings.push(format!("Album {album_id} failed: {}", e.short())),
        }
    }

    for tag_id in &body.tag_ids {
        match immich.set_asset_tag(*tag_id, new_id, true).await {
            Ok(items) => {
                state
                    .tag_counts
                    .invalidate(owner, server_epoch, *tag_id)
                    .await;
                for item in items {
                    if !item.success {
                        warnings.push(format!(
                            "Tag {tag_id} failed: {}",
                            item.error.unwrap_or_else(|| "unknown".into())
                        ));
                    }
                }
            }
            Err(e) => warnings.push(format!("Tag {tag_id} failed: {}", e.short())),
        }
    }

    if body.stack_with_original
        && let Err(e) = stack_with_original(immich, original, new_id, body.stack_primary).await
    {
        warnings.push(format!("Stacking failed: {}", e.short()));
    }

    warnings
}

async fn stack_with_original(
    immich: &crate::immich::ImmichClient,
    original: &AssetDetail,
    new_id: Uuid,
    primary: StackPrimary,
) -> Result<(), crate::immich::ImmichError> {
    let existing_stack_id = original.stack_id.or(original.stack.as_ref().map(|s| s.id));
    let mut ids: Vec<Uuid> = vec![new_id, original.id.source()];
    if let Some(stack_id) = existing_stack_id
        && let Ok(stack) = immich.get_stack(stack_id).await
    {
        for a in stack.assets {
            if !ids.contains(&a.id.source()) {
                ids.push(a.id.source());
            }
        }
    }
    let primary_id = match primary {
        StackPrimary::Edited => new_id,
        StackPrimary::Original => original.id.source(),
    };
    if let Some(pos) = ids.iter().position(|i| *i == primary_id) {
        ids.swap(0, pos);
    }
    let created = immich.create_stack(&ids).await?;
    if created.primary_asset_id != primary_id {
        immich.update_stack_primary(created.id, primary_id).await?;
    }
    Ok(())
}
