use bytes::Bytes;
use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{
    BitDepth, JpegSubsampling, OutputColorSpace, OutputFormat, PngCompression, TiffCompression,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::services::edits_store::{ExportJobRecord, ExportJobStatus};
use crate::services::job_runner::{ItemOutcome, JobExecutor};
use crate::services::job_store::{JobItemRecord, JobRecord};
use crate::services::render::RenderError;
use crate::state::AppState;

pub const EXPORT_MAX_EDGE: u32 = 8192;
pub const DEFAULT_QUALITY: u8 = 90;
pub const EXPORT_JOB_KIND: &str = "export_immich";
pub const DOWNLOAD_ZIP_KIND: &str = "download_zip";

fn default_quality() -> u8 {
    DEFAULT_QUALITY
}

fn default_include_exif() -> bool {
    true
}

pub fn default_suffix() -> String {
    "_edit".into()
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormatKind {
    #[default]
    Jpeg,
    Png,
    Webp,
    Avif,
    Heic,
    Tiff,
    Jxl,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BitDepthOpt {
    #[default]
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "16")]
    Sixteen,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PngCompressionOpt {
    Fast,
    #[default]
    Default,
    Best,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TiffCompressionOpt {
    None,
    #[default]
    Lzw,
    Deflate,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpaceOpt {
    #[default]
    Srgb,
    Displayp3,
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    #[serde(default)]
    pub format: ExportFormatKind,
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default = "default_include_exif")]
    pub include_exif: bool,
    #[serde(default)]
    pub bit_depth: BitDepthOpt,
    #[serde(default)]
    pub png_compression: PngCompressionOpt,
    #[serde(default)]
    pub tiff_compression: TiffCompressionOpt,
    #[serde(default)]
    pub lossless: bool,
    #[serde(default)]
    pub color_space: ColorSpaceOpt,
}

impl Default for ExportParams {
    fn default() -> Self {
        Self {
            format: ExportFormatKind::default(),
            quality: DEFAULT_QUALITY,
            include_exif: true,
            bit_depth: BitDepthOpt::default(),
            png_compression: PngCompressionOpt::default(),
            tiff_compression: TiffCompressionOpt::default(),
            lossless: false,
            color_space: ColorSpaceOpt::default(),
        }
    }
}

impl ExportParams {
    pub fn output_color_space(&self) -> OutputColorSpace {
        match self.color_space {
            ColorSpaceOpt::Srgb => OutputColorSpace::SRgb,
            ColorSpaceOpt::Displayp3 => OutputColorSpace::DisplayP3,
        }
    }

    pub fn output_format(&self) -> OutputFormat {
        let quality = self.quality.clamp(1, 100);
        let bd = match self.bit_depth {
            BitDepthOpt::Eight => BitDepth::Eight,
            BitDepthOpt::Sixteen => BitDepth::Sixteen,
        };
        let png_c = match self.png_compression {
            PngCompressionOpt::Fast => PngCompression::Fast,
            PngCompressionOpt::Default => PngCompression::Default,
            PngCompressionOpt::Best => PngCompression::Best,
        };
        let tiff_c = match self.tiff_compression {
            TiffCompressionOpt::None => TiffCompression::None,
            TiffCompressionOpt::Lzw => TiffCompression::Lzw,
            TiffCompressionOpt::Deflate => TiffCompression::Deflate,
        };
        match self.format {
            ExportFormatKind::Jpeg => OutputFormat::Jpeg {
                quality,
                subsampling: JpegSubsampling::Chroma420,
            },
            ExportFormatKind::Png => OutputFormat::Png {
                bit_depth: bd,
                compression: png_c,
            },
            ExportFormatKind::Webp => OutputFormat::Webp {
                quality,
                lossless: self.lossless || self.include_exif,
            },
            ExportFormatKind::Avif => OutputFormat::Avif { quality },
            ExportFormatKind::Heic => OutputFormat::Heic { quality },
            ExportFormatKind::Tiff => OutputFormat::Tiff {
                bit_depth: bd,
                compression: tiff_c,
            },
            ExportFormatKind::Jxl => OutputFormat::Jxl { bit_depth: bd },
        }
    }
}

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
    pub asset_id: Uuid,
    pub body: &'a ExportToImmichBody,
    pub idempotency_key: Option<String>,
    pub device_asset_id: Option<String>,
}

pub async fn render_export(
    state: &AppState,
    immich: &crate::immich::ImmichClient,
    id: Uuid,
    edits: Edits,
    params: &ExportParams,
) -> Result<(Bytes, OutputFormat), AppError> {
    let frame = state
        .render
        .frame(immich, id)
        .await
        .map_err(map_render_err)?;
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
        .render(immich.clone(), id, edits, opts, None)
        .await
        .map_err(map_render_err)?;

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

    if let (Some(key), Some(hash)) = (idem_key.as_deref(), request_hash.as_deref())
        && let Some(existing) = state.edits.get_export_job(owner, id, key).await?
    {
        if existing.request_hash != hash {
            return Err(AppError::BadRequest(
                "idempotency key reused with different request".into(),
            ));
        }
        if existing.status == ExportJobStatus::Completed {
            return Ok(record_to_result(&existing));
        }
        return resume_export_job(state, immich, owner, id, key, body, existing).await;
    }

    let suffix = validate_suffix(&body.filename_suffix)?;
    let original = immich.asset(id).await?;
    let existing_names = collect_existing_filenames(immich, &original).await;

    let (bytes, output) =
        render_export(state, immich, id, body.edits.clamped(), &body.params).await?;
    let filename = resolve_filename(
        &original.original_file_name,
        &suffix,
        output.extension(),
        &existing_names,
    );
    let device_asset_id = req
        .device_asset_id
        .unwrap_or_else(|| format!("immich-edit-{filename}"));
    let now = Utc::now().to_rfc3339();
    let upload = immich
        .upload_asset(
            &filename,
            output.content_type(),
            bytes,
            body.favorite,
            &now,
            &now,
            &device_asset_id,
        )
        .await?;

    let new_id = upload.id;
    let status = upload.status.clone();

    if let (Some(key), Some(hash)) = (idem_key.as_deref(), request_hash.as_deref()) {
        state
            .edits
            .put_export_job_uploaded(owner, id, key, hash, new_id, &filename, &status)
            .await?;
    }

    let warnings = run_post_upload(state, immich, &original, body, new_id, &status).await;

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

pub fn hash_request(asset_id: Uuid, body: &ExportToImmichBody) -> String {
    let mut album_ids = body.album_ids.clone();
    album_ids.sort();
    let mut tag_ids = body.tag_ids.clone();
    tag_ids.sort();
    let canonical = serde_json::json!({
        "asset_id": asset_id,
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

async fn resume_export_job(
    state: &AppState,
    immich: &crate::immich::ImmichClient,
    owner: Uuid,
    asset_id: Uuid,
    key: &str,
    body: &ExportToImmichBody,
    existing: ExportJobRecord,
) -> Result<ExportToImmichResult, AppError> {
    let Some(new_id) = existing.immich_asset_id else {
        return Err(AppError::Internal);
    };
    let original = immich.asset(asset_id).await?;
    let upload_status = existing.upload_status.clone().unwrap_or_default();
    let warnings = run_post_upload(state, immich, &original, body, new_id, &upload_status).await;
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

async fn run_post_upload(
    state: &AppState,
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
        warnings.push(format!("Favorite failed: {}", short_err(&e)));
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
            Err(e) => warnings.push(format!("Album {album_id} failed: {}", short_err(&e))),
        }
    }

    for tag_id in &body.tag_ids {
        match immich.tag_asset(*tag_id, new_id).await {
            Ok(items) => {
                state.tag_counts.invalidate(*tag_id).await;
                for item in items {
                    if !item.success {
                        warnings.push(format!(
                            "Tag {tag_id} failed: {}",
                            item.error.unwrap_or_else(|| "unknown".into())
                        ));
                    }
                }
            }
            Err(e) => warnings.push(format!("Tag {tag_id} failed: {}", short_err(&e))),
        }
    }

    if body.stack_with_original
        && let Err(e) = stack_with_original(immich, original, new_id, body.stack_primary).await
    {
        warnings.push(format!("Stacking failed: {}", short_err(&e)));
    }

    warnings
}

pub fn validate_suffix(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok("_edit".into());
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | '\0'))
    {
        return Err(AppError::BadRequest("invalid filename suffix".into()));
    }
    if trimmed.len() > 32 {
        return Err(AppError::BadRequest("filename suffix too long".into()));
    }
    Ok(trimmed.to_string())
}

async fn collect_existing_filenames(
    immich: &crate::immich::ImmichClient,
    original: &AssetDetail,
) -> Vec<String> {
    let mut names = vec![original.original_file_name.clone()];
    let Some(stack_id) = original.stack_id.or(original.stack.as_ref().map(|s| s.id)) else {
        return names;
    };
    match immich.get_stack(stack_id).await {
        Ok(stack) => {
            for asset in stack.assets {
                names.push(asset.original_file_name);
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "fetch stack for filename collision");
        }
    }
    names
}

pub fn resolve_filename(
    original: &str,
    suffix: &str,
    extension: &str,
    existing: &[String],
) -> String {
    let stem = match original.rsplit_once('.') {
        Some((s, _)) => s,
        None => original,
    };
    let lower: HashSet<String> = existing.iter().map(|n| n.to_ascii_lowercase()).collect();
    let mut n: u32 = 1;
    loop {
        let candidate = if n == 1 {
            format!("{stem}{suffix}.{extension}")
        } else {
            format!("{stem}{suffix}_{n}.{extension}")
        };
        if !lower.contains(&candidate.to_ascii_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

async fn stack_with_original(
    immich: &crate::immich::ImmichClient,
    original: &AssetDetail,
    new_id: Uuid,
    primary: StackPrimary,
) -> Result<(), crate::immich::ImmichError> {
    let existing_stack_id = original.stack_id.or(original.stack.as_ref().map(|s| s.id));
    let mut ids: Vec<Uuid> = vec![new_id, original.id];
    if let Some(stack_id) = existing_stack_id
        && let Ok(stack) = immich.get_stack(stack_id).await
    {
        for a in stack.assets {
            if !ids.contains(&a.id) {
                ids.push(a.id);
            }
        }
    }
    let primary_id = match primary {
        StackPrimary::Edited => new_id,
        StackPrimary::Original => original.id,
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

fn short_err(err: &crate::immich::ImmichError) -> String {
    match err {
        crate::immich::ImmichError::Unauthorized => "unauthorized".into(),
        crate::immich::ImmichError::NotFound => "not found".into(),
        crate::immich::ImmichError::Timeout => "timeout".into(),
        crate::immich::ImmichError::Status(c) => format!("status {c}"),
        crate::immich::ImmichError::Transport(_) => "transport error".into(),
        crate::immich::ImmichError::Decode(_) => "decode error".into(),
    }
}

pub fn map_render_err(err: RenderError) -> AppError {
    match err {
        RenderError::Upstream(e) => e.into(),
        RenderError::Pipeline(raw_pipeline::PipelineError::Unsupported(msg)) => {
            AppError::UnsupportedFormat(msg)
        }
        RenderError::Pipeline(e) => {
            tracing::error!(error = %e, "export render");
            AppError::Internal
        }
        RenderError::Lut(m) => AppError::BadRequest(m),
        RenderError::Dcp(m) => AppError::BadRequest(m),
    }
}

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
            let asset_id =
                Uuid::parse_str(&item.asset_id).map_err(|_| "invalid asset id".to_string())?;
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
                other => Err(format!("unsupported job kind: {other}")),
            }
        })
    }
}

fn parse_job_params(job: &JobRecord) -> Result<ExportJobParams, String> {
    serde_json::from_value(job.params.clone()).map_err(|e| format!("invalid export params: {e}"))
}

async fn job_edits(
    state: &AppState,
    owner: Uuid,
    params: &ExportJobParams,
    asset_id: Uuid,
) -> Result<Edits, String> {
    match &params.manifest {
        Some(manifest) => Ok(manifest.to_edits()),
        None => state
            .edits
            .get_edits_or_default(owner, asset_id)
            .await
            .map_err(|e| e.to_string()),
    }
}

async fn run_immich_item(state: &AppState, job: &JobRecord, asset_id: Uuid) -> ItemOutcome {
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
        &state.immich,
        job.user_id,
        ExportImmichRequest {
            asset_id,
            body: &body,
            idempotency_key: Some(idempotency_key),
            device_asset_id: Some(device_asset_id),
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

async fn run_zip_item(state: &AppState, job: &JobRecord, asset_id: Uuid) -> ItemOutcome {
    let params = parse_job_params(job)?;
    let edits = job_edits(state, job.user_id, &params, asset_id).await?;
    let suffix = validate_suffix(&params.filename_suffix).map_err(|e| e.to_string())?;
    let original = state
        .immich
        .asset(asset_id)
        .await
        .map_err(|e| e.to_string())?;
    let (bytes, output) = render_export(
        state,
        &state.immich,
        asset_id,
        edits.clamped(),
        &params.params,
    )
    .await
    .map_err(|e| e.to_string())?;
    let dir = zip_job_dir(state, job.id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("create export dir: {e}"))?;
    let base = sanitize_filename(&original.original_file_name);
    let filename = write_unique(&dir, &base, &suffix, output.extension(), &bytes)
        .await
        .map_err(|e| format!("write export file: {e}"))?;
    Ok(serde_json::json!({ "filename": filename, "bytes": bytes.len() }))
}

pub fn zip_job_dir(state: &AppState, job_id: Uuid) -> PathBuf {
    state
        .config
        .cache_dir
        .join("exports")
        .join(job_id.to_string())
}

pub fn zip_archive_path(state: &AppState, job_id: Uuid) -> PathBuf {
    state
        .config
        .cache_dir
        .join("exports")
        .join(format!("{job_id}.zip"))
}

pub async fn cleanup_zip_job(state: &AppState, job_id: Uuid) {
    let dir = zip_job_dir(state, job_id);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "remove export dir");
    }
    let archive = zip_archive_path(state, job_id);
    if let Err(e) = tokio::fs::remove_file(&archive).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "remove export archive");
    }
}

fn sanitize_filename(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((s, _)) => s,
        None => name,
    };
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | '\0' | ':') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "export".into()
    } else {
        trimmed.to_string()
    }
}

async fn write_unique(
    dir: &Path,
    stem: &str,
    suffix: &str,
    extension: &str,
    bytes: &[u8],
) -> std::io::Result<String> {
    let mut n: u32 = 1;
    loop {
        let filename = if n == 1 {
            format!("{stem}{suffix}.{extension}")
        } else {
            format!("{stem}{suffix}_{n}.{extension}")
        };
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(&filename))
            .await
        {
            Ok(mut file) => {
                use tokio::io::AsyncWriteExt;
                file.write_all(bytes).await?;
                file.flush().await?;
                return Ok(filename);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                n += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

pub async fn build_zip_archive(state: &AppState, job_id: Uuid) -> Result<PathBuf, AppError> {
    let archive = zip_archive_path(state, job_id);
    if tokio::fs::try_exists(&archive).await.unwrap_or(false) {
        return Ok(archive);
    }
    let dir = zip_job_dir(state, job_id);
    let archive_for_task = archive.clone();
    tokio::task::spawn_blocking(move || zip_dir_blocking(&dir, &archive_for_task))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "zip task join");
            AppError::Internal
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "build zip");
            AppError::Internal
        })?;
    Ok(archive)
}

fn zip_dir_blocking(dir: &Path, archive: &Path) -> std::io::Result<()> {
    let tmp = archive.with_extension("zip.part");
    let file = std::fs::File::create(&tmp)?;
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        writer.start_file(name, options)?;
        let mut src = std::fs::File::open(entry.path())?;
        std::io::copy(&mut src, &mut writer)?;
    }
    let mut out = writer.finish()?;
    out.flush()?;
    std::fs::rename(&tmp, archive)?;
    Ok(())
}
