use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::lens_profile::{self, LensProfileMatch};
use crate::routes::auth::AuthCtx;
use crate::state::AppState;

#[derive(Serialize)]
pub struct LensProfileResponse {
    #[serde(flatten)]
    profile: LensProfileMatch,
    auto: bool,
}

pub async fn get_lens_profile(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<LensProfileResponse>, AppError> {
    let asset = ctx.immich.asset(id.source()).await?;
    let profile = asset
        .exif_info
        .as_ref()
        .map(lens_profile::lookup)
        .unwrap_or_default();
    Ok(Json(LensProfileResponse {
        profile,
        auto: state.render.lens_auto(),
    }))
}
