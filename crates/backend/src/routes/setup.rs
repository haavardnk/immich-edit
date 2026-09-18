use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::routes::auth;
use crate::services::login_limiter::LoginKey;
use crate::state::AppState;

pub async fn status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let cfg = state.instance.get().await?;
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
    client: auth::ClientMeta,
    headers: HeaderMap,
    Json(body): Json<SetupBody>,
) -> Result<Response, AppError> {
    let cfg = state.instance.get().await?;
    if cfg.is_configured() {
        return Err(AppError::Conflict("instance already configured".into()));
    }
    let key = match body.email.as_deref() {
        Some(email) => LoginKey::identity("setup", &client.ip, email),
        None => LoginKey::client("setup", &client.ip),
    };
    if let Some(duration) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(duration.as_secs())));
    }
    let base = auth::validate_candidate_url(&body.immich_url)?;

    let validated = auth::validate_credentials(
        &base,
        body.email.as_deref(),
        body.password.as_deref(),
        body.api_key.as_deref(),
    )
    .await;
    let (user, kind, cred) = match validated {
        Ok(value) => value,
        Err(error) => {
            state.login_limiter.record_failure(&key);
            return Err(error);
        }
    };

    if !user.is_admin {
        state.login_limiter.record_failure(&key);
        return Err(AppError::AdminRequired);
    }
    state.login_limiter.record_success(&key);

    auth::finish_setup(&state, base.as_str(), &user, kind, &cred, &headers, &client).await
}
