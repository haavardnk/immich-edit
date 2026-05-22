use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{BitDepth, OutputFormat, PngCompression, TiffCompression};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
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

async fn export(
    state: AppState,
    id: Uuid,
    edits: Edits,
    params: ExportParams,
) -> Result<Response, AppError> {
    let frame = state.render.frame(id).await.map_err(map_render_err)?;
    let output = params.output_format();
    let opts = raw_pipeline::frame::RenderOptions {
        max_edge: EXPORT_MAX_EDGE,
        quality: true,
        output,
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
