use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::edits_store::Appearance;
use crate::state::AppState;

const MAX_ID_LEN: usize = 64;
const MAX_ACCENTS: usize = 16;

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn validate(appearance: &Appearance) -> Result<(), AppError> {
    if !valid_id(&appearance.light_theme) {
        return Err(AppError::BadRequest("invalid light theme id".into()));
    }
    if !valid_id(&appearance.dark_theme) {
        return Err(AppError::BadRequest("invalid dark theme id".into()));
    }
    if appearance.accents.len() > MAX_ACCENTS {
        return Err(AppError::BadRequest("too many accents".into()));
    }
    if !appearance
        .accents
        .iter()
        .all(|(family, accent)| valid_id(family) && valid_id(accent))
    {
        return Err(AppError::BadRequest("invalid accent".into()));
    }
    Ok(())
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Appearance>, AppError> {
    Ok(Json(
        state
            .edits
            .get_appearance(ctx.owner)
            .await?
            .unwrap_or_default(),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<Appearance>,
) -> Result<Json<Appearance>, AppError> {
    validate(&body)?;
    Ok(Json(state.edits.put_appearance(ctx.owner, &body).await?))
}
