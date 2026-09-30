use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::render::{RenderIdentity, WARM_NEIGHBOURS};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct WarmBody {
    ids: Vec<AssetKey>,
}

pub async fn warm(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<WarmBody>,
) -> Result<StatusCode, AppError> {
    if body.ids.is_empty() || body.ids.len() > WARM_NEIGHBOURS {
        return Err(AppError::BadRequest(format!(
            "warm takes 1 to {WARM_NEIGHBOURS} assets"
        )));
    }
    let mut sources: Vec<Uuid> = body.ids.iter().map(AssetKey::source).collect();
    sources.dedup();
    let identity = RenderIdentity::from(&ctx);
    if state.render.warm(identity, ctx.immich, sources).await {
        Ok(StatusCode::ACCEPTED)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
