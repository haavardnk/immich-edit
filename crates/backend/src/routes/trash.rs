use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::trash;
use crate::state::AppState;

const MAX_IDS: usize = 1000;

#[derive(Debug, Deserialize)]
pub struct TrashBody {
    pub ids: Vec<AssetKey>,
}

#[derive(Debug, Serialize)]
pub struct TrashResult {
    pub count: u64,
}

pub async fn trash(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<TrashBody>,
) -> Result<Json<TrashResult>, AppError> {
    let ids = master_ids(&body.ids)?;
    trash::move_to_trash(&state, (&ctx).into(), &ctx.immich, &ids).await?;
    Ok(Json(TrashResult {
        count: ids.len() as u64,
    }))
}

pub async fn restore(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<TrashBody>,
) -> Result<Json<TrashResult>, AppError> {
    let ids = master_ids(&body.ids)?;
    let count = trash::restore(&state, (&ctx).into(), &ctx.immich, &ids).await?;
    Ok(Json(TrashResult { count }))
}

fn master_ids(keys: &[AssetKey]) -> Result<Vec<Uuid>, AppError> {
    if keys.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if keys.len() > MAX_IDS {
        return Err(AppError::BadRequest(format!(
            "at most {MAX_IDS} assets per request"
        )));
    }
    if keys.iter().any(AssetKey::is_copy) {
        return Err(AppError::BadRequest(
            "virtual copies are deleted in immich-edit, not trashed in Immich".into(),
        ));
    }
    let mut ids: Vec<Uuid> = keys.iter().map(AssetKey::source).collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}
