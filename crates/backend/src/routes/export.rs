use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use chrono::Utc;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{BitDepth, OutputFormat, PngCompression, TiffCompression};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetDetail;
use crate::services::render::RenderError;
use crate::state::AppState;

const EXPORT_MAX_EDGE: u32 = 8192;
const DEFAULT_QUALITY: u8 = 90;
const DEFAULT_AVIF_SPEED: u8 = 6;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormatKind {
    #[default]
    Jpeg,
    Png,
    Webp,
    Avif,
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

fn default_quality() -> u8 {
    DEFAULT_QUALITY
}

fn default_include_exif() -> bool {
    true
}

fn default_speed() -> u8 {
    DEFAULT_AVIF_SPEED
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
    #[serde(default = "default_speed")]
    pub speed: u8,
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
            speed: DEFAULT_AVIF_SPEED,
        }
    }
}

impl ExportParams {
    fn output_format(&self) -> OutputFormat {
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
            ExportFormatKind::Jpeg => OutputFormat::Jpeg { quality },
            ExportFormatKind::Png => OutputFormat::Png {
                bit_depth: bd,
                compression: png_c,
            },
            ExportFormatKind::Webp => OutputFormat::Webp {
                quality,
                lossless: self.lossless || self.include_exif,
            },
            ExportFormatKind::Avif => OutputFormat::Avif {
                quality,
                speed: self.speed.clamp(1, 10),
            },
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

pub async fn get_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ExportParams>,
) -> Result<Response, AppError> {
    let edits = state.edits.get_edits_or_default(id).await.map_err(|e| {
        tracing::error!(error = %e, "edits store");
        AppError::Internal
    })?;
    export(state, id, edits, params).await
}

pub async fn post_export(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExportBody>,
) -> Result<Response, AppError> {
    export(state, id, body.edits.clamped(), body.params).await
}

async fn render_export(
    state: &AppState,
    id: Uuid,
    edits: Edits,
    params: &ExportParams,
) -> Result<(Bytes, OutputFormat), AppError> {
    let frame = state.render.frame(id).await.map_err(map_render_err)?;
    let output = params.output_format();
    let opts = raw_pipeline::frame::RenderOptions {
        max_edge: EXPORT_MAX_EDGE,
        quality: true,
        output,
        ..Default::default()
    };
    let rendered = state
        .render
        .render(id, edits, opts, None)
        .await
        .map_err(map_render_err)?;

    let mut bytes = rendered.bytes;
    if params.include_exif {
        if let Some(exif) = frame.exif.as_ref() {
            if let Err(e) =
                raw_pipeline::exif::inject(&mut bytes, exif, output.exif_file_extension())
            {
                tracing::warn!(error = %e, "exif inject failed");
            }
        }
    }
    Ok((Bytes::from(bytes), output))
}

async fn export(
    state: AppState,
    id: Uuid,
    edits: Edits,
    params: ExportParams,
) -> Result<Response, AppError> {
    let (bytes, output) = render_export(&state, id, edits, &params).await?;
    let content_type = output.content_type();
    let extension = output.extension();
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{id}.{extension}\"")).unwrap(),
    );
    Ok(resp.into_response())
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StackPrimary {
    #[default]
    Edited,
    Original,
}

fn default_suffix() -> String {
    "_edit".into()
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

pub async fn post_export_immich(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExportToImmichBody>,
) -> Result<Json<ExportToImmichResult>, AppError> {
    let suffix = validate_suffix(&body.filename_suffix)?;
    let original = state.immich.asset(id).await?;
    let existing_names = collect_existing_filenames(&state, &original).await;

    let (bytes, output) = render_export(&state, id, body.edits.clamped(), &body.params).await?;
    let filename = resolve_filename(
        &original.original_file_name,
        &suffix,
        output.extension(),
        &existing_names,
    );
    let now = Utc::now().to_rfc3339();
    let upload = state
        .immich
        .upload_asset(
            &filename,
            output.content_type(),
            bytes,
            body.favorite,
            &now,
            &now,
        )
        .await?;

    let mut warnings: Vec<String> = Vec::new();
    let new_id = upload.id;
    let status = upload.status.clone();
    let is_duplicate = status.eq_ignore_ascii_case("duplicate");

    if body.favorite && is_duplicate {
        if let Err(e) = state
            .immich
            .update_asset(new_id, &serde_json::json!({ "isFavorite": true }))
            .await
        {
            warnings.push(format!("Favorite failed: {}", short_err(&e)));
        }
    }

    for album_id in &body.album_ids {
        match state.immich.add_assets_to_album(*album_id, &[new_id]).await {
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
        match state.immich.tag_asset(*tag_id, new_id).await {
            Ok(items) => {
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

    if body.stack_with_original {
        if let Err(e) = stack_with_original(&state, &original, new_id, body.stack_primary).await {
            warnings.push(format!("Stacking failed: {}", short_err(&e)));
        }
    }

    Ok(Json(ExportToImmichResult {
        asset_id: new_id,
        filename,
        status,
        warnings,
    }))
}

fn validate_suffix(raw: &str) -> Result<String, AppError> {
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

async fn collect_existing_filenames(state: &AppState, original: &AssetDetail) -> Vec<String> {
    let mut names = vec![original.original_file_name.clone()];
    let Some(stack_id) = original.stack_id.or(original.stack.as_ref().map(|s| s.id)) else {
        return names;
    };
    match state.immich.get_stack(stack_id).await {
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
    state: &AppState,
    original: &AssetDetail,
    new_id: Uuid,
    primary: StackPrimary,
) -> Result<(), crate::immich::ImmichError> {
    let existing_stack_id = original.stack_id.or(original.stack.as_ref().map(|s| s.id));
    let mut ids: Vec<Uuid> = vec![new_id, original.id];
    if let Some(stack_id) = existing_stack_id {
        if let Ok(stack) = state.immich.get_stack(stack_id).await {
            for a in stack.assets {
                if !ids.contains(&a.id) {
                    ids.push(a.id);
                }
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
    let created = state.immich.create_stack(&ids).await?;
    if created.primary_asset_id != primary_id {
        state
            .immich
            .update_stack_primary(created.id, primary_id)
            .await?;
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

fn map_render_err(err: RenderError) -> AppError {
    match err {
        RenderError::Upstream(e) => e.into(),
        RenderError::Pipeline(raw_pipeline::PipelineError::Unsupported(msg)) => {
            AppError::UnsupportedFormat(msg)
        }
        RenderError::Pipeline(e) => {
            tracing::error!(error = %e, "export render");
            AppError::Internal
        }
    }
}
