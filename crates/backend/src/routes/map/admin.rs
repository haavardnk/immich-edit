use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::routes::admin::require_fresh_admin;
use crate::routes::auth::AdminCtx;
use crate::services::map::MapSettings;
use crate::state::AppState;

pub async fn settings(State(state): State<AppState>, _admin: AdminCtx) -> Json<MapSettings> {
    Json(state.map.settings())
}

pub async fn update_settings(
    State(state): State<AppState>,
    admin: AdminCtx,
    Json(body): Json<MapSettings>,
) -> Result<Json<MapSettings>, AppError> {
    require_fresh_admin(&admin).await?;
    let settings = body.validated().map_err(AppError::BadRequest)?;
    state.instance.set_map_settings(&settings).await?;
    state.map.set_settings(settings.clone());
    Ok(Json(settings))
}
