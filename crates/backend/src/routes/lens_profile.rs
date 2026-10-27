use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::lens_profile::{self, LensProfileMatch};
use crate::routes::auth::AuthCtx;
use crate::services::render::RenderIdentity;
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
    let embedded = is_dng(&asset.original_file_name)
        && state
            .render
            .frame(RenderIdentity::from(&ctx), &ctx.immich, id.source())
            .await
            .is_ok_and(|frame| frame.embedded_lens.is_corrected());
    Ok(Json(LensProfileResponse {
        profile,
        auto: state.render.lens_auto() && !embedded,
    }))
}

fn is_dng(file_name: &str) -> bool {
    file_name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("dng"))
}
