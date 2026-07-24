use axum::Json;
use axum::extract::Path;
use uuid::Uuid;

use crate::error::AppError;
use crate::lens_profile::{self, LensProfileMatch};
use crate::routes::auth::AuthCtx;

pub async fn get_lens_profile(
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Json<LensProfileMatch>, AppError> {
    let asset = ctx.immich.asset(id).await?;
    let Some(exif) = asset.exif_info.as_ref() else {
        return Ok(Json(LensProfileMatch::default()));
    };
    Ok(Json(lens_profile::lookup(exif)))
}
