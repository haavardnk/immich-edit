use axum::Json;
use axum::extract::Path;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::lens_profile::{self, LensProfileMatch};
use crate::routes::auth::AuthCtx;

pub async fn get_lens_profile(
    ctx: AuthCtx,
    Path(id): Path<AssetKey>,
) -> Result<Json<LensProfileMatch>, AppError> {
    let asset = ctx.immich.asset(id.source()).await?;
    let Some(exif) = asset.exif_info.as_ref() else {
        return Ok(Json(LensProfileMatch::default()));
    };
    Ok(Json(lens_profile::lookup(exif)))
}
