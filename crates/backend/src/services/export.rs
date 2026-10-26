use bytes::Bytes;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::OutputFormat;
use serde::Deserialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::services::render::RenderIdentity;
use crate::services::render_queue::RenderPriority;
use crate::services::watermark_store::WatermarkStoreError;
use crate::state::AppState;

pub const EXPORT_MAX_EDGE: u32 = 65535;
pub const DEFAULT_QUALITY: u8 = 90;
pub const EXPORT_JOB_KIND: &str = "export_immich";
pub const DOWNLOAD_ZIP_KIND: &str = "download_zip";

mod archive;
mod batch;
mod denoise;
mod immich;
mod immich_metadata;
mod naming;
mod output_sharpen;
mod params;
mod post_upload;
mod resize;
mod watermark;

pub use archive::*;
pub use batch::*;
pub use immich::*;
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

pub struct RenderedExport {
    pub bytes: Bytes,
    pub output: OutputFormat,
    pub warnings: Vec<String>,
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
    let denoise = denoise::prepare(state, identity, immich, id, &edits).await?;
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
            denoise: denoise.plane,
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
        let warnings = denoise
            .skipped
            .map(str::to_string)
            .into_iter()
            .chain(rendered.metadata_warnings)
            .collect();
        Ok(RenderedExport {
            bytes: Bytes::from(rendered.bytes),
            output,
            warnings,
        })
    };
    state
        .queue
        .run(priority, work)
        .await
        .ok_or(AppError::Internal)?
}
