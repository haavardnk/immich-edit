use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::{Preferences, PreferencesPatch};
use crate::state::AppState;

mod validate;

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Preferences>, AppError> {
    Ok(Json(
        state
            .edits
            .get_preferences(ctx.owner)
            .await?
            .unwrap_or_default(),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(patch): Json<PreferencesPatch>,
) -> Result<Json<Preferences>, AppError> {
    validate::patch(&patch).map_err(AppError::BadRequest)?;
    Ok(Json(
        state.edits.update_preferences(ctx.owner, patch).await?,
    ))
}
