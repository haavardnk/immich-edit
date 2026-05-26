use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::lens_profile::{self, LensProfileMatch};
use crate::state::AppState;

pub async fn get_lens_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LensProfileMatch>, AppError> {
    let asset = state.immich.asset(id).await?;
    let Some(exif) = asset.exif_info.as_ref() else {
        return Ok(Json(LensProfileMatch::default()));
    };
    Ok(Json(lens_profile::lookup(exif)))
}
