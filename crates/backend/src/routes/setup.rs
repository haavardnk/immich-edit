use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::routes::auth;
use crate::services::auth_store::AuthKind;
use crate::state::AppState;

pub async fn status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let cfg = state.instance.get().await.map_err(|_| AppError::Internal)?;
    Ok(Json(json!({ "configured": cfg.is_configured() })))
}

#[derive(Deserialize)]
pub struct SetupBody {
    pub immich_url: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub api_key: Option<String>,
}

pub async fn complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SetupBody>,
) -> Result<Response, AppError> {
    let cfg = state.instance.get().await.map_err(|_| AppError::Internal)?;
    if cfg.is_configured() {
        return Err(AppError::Conflict("instance already configured".into()));
    }
    let base = auth::validate_candidate_url(&body.immich_url)?;

    let (user, kind, cred): (_, _, Vec<u8>) = if let Some(api_key) = body.api_key.as_deref() {
        let user = auth::validate_api_key(&base, api_key).await?;
        (user, AuthKind::ApiKey, api_key.as_bytes().to_vec())
    } else if let (Some(email), Some(password)) = (body.email.as_deref(), body.password.as_deref())
    {
        let (user, cred) = auth::validate_password(&base, email, password).await?;
        (user, AuthKind::Password, cred)
    } else {
        return Err(AppError::BadRequest(
            "email+password or api_key required".into(),
        ));
    };

    if !user.is_admin {
        return Err(AppError::AdminRequired);
    }

    state
        .instance
        .claim(base.as_str())
        .await
        .map_err(|_| AppError::Conflict("instance already configured".into()))?;

    let ip = auth::client_ip(&headers);
    auth::finish_login(&state, &user, kind, &cred, &headers, Some(ip.as_str())).await
}
