use std::sync::Arc;

use raw_pipeline::ai_denoise::DenoisePlane;
use raw_pipeline::edits::Edits;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

const ML_DISABLED: &str = "AI denoise skipped: machine learning is disabled on this server.";
#[cfg(feature = "ml")]
const MODEL_MISSING: &str = "AI denoise skipped: no denoise model is installed.";
#[cfg(feature = "ml")]
const UNSUPPORTED: &str = "AI denoise skipped: it needs a RAW photo with usable color data.";

#[derive(Default)]
pub struct PreparedDenoise {
    pub plane: Option<Arc<DenoisePlane>>,
    pub skipped: Option<&'static str>,
}

impl PreparedDenoise {
    fn skipped(id: AssetKey, reason: &'static str) -> Self {
        tracing::warn!(asset = %id, reason, "exporting without AI denoise");
        Self {
            plane: None,
            skipped: Some(reason),
        }
    }
}

#[cfg(feature = "ml")]
pub async fn prepare(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    id: AssetKey,
    edits: &Edits,
) -> Result<PreparedDenoise, AppError> {
    use crate::services::denoise::ExportPlane;

    if !edits.detail.clamped().ai_denoise_active() {
        return Ok(PreparedDenoise::default());
    }
    let plane = state
        .denoise
        .export_plane(identity, immich, id.source())
        .await?;
    Ok(match plane {
        ExportPlane::Ready(plane) => PreparedDenoise {
            plane: Some(plane),
            skipped: None,
        },
        ExportPlane::Disabled => PreparedDenoise::skipped(id, ML_DISABLED),
        ExportPlane::ModelMissing => PreparedDenoise::skipped(id, MODEL_MISSING),
        ExportPlane::Unsupported => PreparedDenoise::skipped(id, UNSUPPORTED),
    })
}

#[cfg(not(feature = "ml"))]
pub async fn prepare(
    _state: &AppState,
    _identity: RenderIdentity,
    _immich: &ImmichClient,
    id: AssetKey,
    edits: &Edits,
) -> Result<PreparedDenoise, AppError> {
    Ok(if edits.detail.clamped().ai_denoise_active() {
        PreparedDenoise::skipped(id, ML_DISABLED)
    } else {
        PreparedDenoise::default()
    })
}
