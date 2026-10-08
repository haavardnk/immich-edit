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
use crate::services::edits_store::{ExportJobKey, ExportJobRecord, ExportJobStatus};
use crate::services::render::RenderIdentity;
use crate::services::render_queue::RenderPriority;
use crate::services::watermark_store::WatermarkStoreError;
use crate::state::AppState;
use crate::telemetry::ErrorChain;

pub const EXPORT_MAX_EDGE: u32 = 65535;
pub const DEFAULT_QUALITY: u8 = 90;
pub const EXPORT_JOB_KIND: &str = "export_immich";
pub const DOWNLOAD_ZIP_KIND: &str = "download_zip";

mod archive;
mod batch;
mod immich_metadata;
mod naming;
mod output_sharpen;
mod params;
mod resize;
mod watermark;

pub use archive::*;
pub use batch::*;
pub use naming::*;
pub use params::*;
pub use resize::{Resize, ResizeMode};

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

pub struct RenderedExport {
    pub bytes: Bytes,
    pub output: OutputFormat,
    pub metadata_warnings: Vec<String>,
}

pub async fn render_export(
    state: &AppState,
    identity: RenderIdentity,
    immich: &crate::immich::ImmichClient,
    id: AssetKey,
    edits: Edits,
    params: &ExportParams,
    priority: RenderPriority,
) -> Result<RenderedExport, AppError> {
    let resize = params.resize()?;
    let output_sharpen = params.output_sharpen()?;
    let watermark = match params.watermark()? {
        Some(placement) => {
            let image = state
                .watermarks
                .load(&placement.id)
                .await
                .map_err(|e| match e {
                    WatermarkStoreError::NotFound => {
                        AppError::BadRequest(format!("unknown watermark: {}", placement.id))
                    }
                    e => e.into(),
                })?;
            Some(placement.with_image(image))
        }
        None => None,
    };
    let work = async {
        let frame = state
            .render
            .frame(identity, immich, id.source())
            .await
            .map_err(AppError::from)?;
        let output = params.output_format();
        let crop = raw_pipeline::geom::display_crop_px(
            frame.meta.orientation,
            &edits.clamped(),
            (frame.meta.width as u32, frame.meta.height as u32),
        );
        let edge = resize.map(|r| r.output_edge(crop));
        let metadata = match params.metadata() {
            MetadataOpt::All => Some(true),
            MetadataOpt::NoLocation => Some(false),
            MetadataOpt::None => None,
        }
        .map(|location| raw_pipeline::metadata::ExportMetadata {
            exif: frame.exif.clone(),
            location,
        });
        let opts = raw_pipeline::frame::RenderOptions {
            max_edge: edge.map_or(EXPORT_MAX_EDGE, |e| e.max_edge),
            enlarge: edge.is_some_and(|e| e.enlarge),
            output_sharpen,
            watermark,
            quality: true,
            output,
            output_color_space: params.output_color_space(),
            metadata,
            ..Default::default()
        };
        let rendered = state
            .render
            .render(identity, immich.clone(), id.source(), edits, opts, None)
            .await
            .map_err(AppError::from)?;
        for warning in &rendered.metadata_warnings {
            tracing::warn!(asset = %id, warning = %warning, "export metadata not copied");
        }
        Ok(RenderedExport {
            bytes: Bytes::from(rendered.bytes),
            output,
            metadata_warnings: rendered.metadata_warnings,
        })
    };
    state
        .queue
        .run(priority, work)
        .await
        .ok_or(AppError::Internal)?
}

pub async fn export_to_immich(
    state: &AppState,
    immich: &crate::immich::ImmichClient,
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
            .upload_asset(crate::immich::client::UploadRequest {
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
                .put_export_job_uploaded(
                    job,
                    hash,
                    new_id,
                    &filename,
                    &status,
                    &rendered.metadata_warnings,
                )
                .await?;
        }

        let warnings: Vec<String> = rendered
            .metadata_warnings
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
    immich: &crate::immich::ImmichClient,
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

async fn run_post_upload(
    state: &AppState,
    identity: RenderIdentity,
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

    if let Some(update) =
        immich_metadata::bulk_update(new_id, original.exif_info.as_ref(), body.params.metadata())
    {
        immich_metadata::await_extraction(immich, new_id).await;
        if let Err(e) = immich.update_assets(&update).await {
            warnings.push(format!("Metadata copy failed: {}", e.short()));
        }
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
                    .invalidate(identity.owner, identity.server_epoch, *tag_id)
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

    if body.stack_with_original {
        if original
            .owner_id
            .is_some_and(|owner| owner != identity.owner)
        {
            warnings.push("Stacking skipped: the original belongs to another Immich user".into());
        } else if let Err(e) =
            stack_with_original(immich, original, new_id, body.stack_primary).await
        {
            warnings.push(format!("Stacking failed: {}", e.short()));
        }
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
