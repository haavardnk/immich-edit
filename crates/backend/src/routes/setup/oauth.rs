use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::error::AppError;
use crate::routes::auth;
use crate::routes::auth::oauth;
use crate::services::auth_store::AuthKind;
use crate::services::login_limiter::LoginKey;
use crate::services::oauth_flow::{FlowPurpose, OAuthFlow};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ProvidersQuery {
    pub immich_url: String,
}

#[derive(Deserialize)]
pub struct StartBody {
    pub immich_url: String,
    pub redirect_uri: String,
}

#[derive(Deserialize)]
pub struct CompleteBody {
    pub url: String,
}

async fn require_unconfigured(state: &AppState) -> Result<(), AppError> {
    let cfg = state.instance.get().await?;
    if cfg.is_configured() {
        return Err(AppError::Conflict("instance already configured".into()));
    }
    Ok(())
}

pub async fn providers(
    State(state): State<AppState>,
    Query(query): Query<ProvidersQuery>,
) -> Result<Response, AppError> {
    require_unconfigured(&state).await?;
    let base = auth::validate_candidate_url(&query.immich_url)?;
    let p = state.providers.get(&base).await;
    Ok((
        StatusCode::OK,
        Json(json!({
            "oauth": p.oauth,
            "password_login": p.password_login,
            "auto_launch": p.auto_launch,
            "button_text": p.button_text,
            "degraded": p.degraded,
        })),
    )
        .into_response())
}

pub async fn start(
    State(state): State<AppState>,
    client: auth::ClientMeta,
    headers: HeaderMap,
    Json(body): Json<StartBody>,
) -> Result<Response, AppError> {
    require_unconfigured(&state).await?;
    let base = auth::validate_candidate_url(&body.immich_url)?;
    let redirect = oauth::validate_redirect_uri(&body.redirect_uri, &headers)?;
    let flow = OAuthFlow::begin(
        FlowPurpose::Setup,
        &redirect,
        None,
        Some(base.as_str().to_string()),
    );
    oauth::begin_flow(&base, &flow, &state.crypto, client.secure).await
}

pub async fn complete(
    State(state): State<AppState>,
    client: auth::ClientMeta,
    headers: HeaderMap,
    Json(body): Json<CompleteBody>,
) -> Result<Response, AppError> {
    require_unconfigured(&state).await?;
    let key = LoginKey::client("setup-oauth", &client.ip);
    if let Some(duration) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(duration.as_secs())));
    }
    match exchange(&state, &client, &headers, &body).await {
        Ok(resp) => {
            state.login_limiter.record_success(&key);
            Ok(resp)
        }
        Err(e) => {
            state.login_limiter.record_failure(&key);
            Err(e)
        }
    }
}

async fn exchange(
    state: &AppState,
    client: &auth::ClientMeta,
    headers: &HeaderMap,
    body: &CompleteBody,
) -> Result<Response, AppError> {
    let flow = oauth::open_flow(state, headers, FlowPurpose::Setup)?;
    let callback_url = oauth::validate_callback_url(&body.url, &flow)?;
    let raw_base = flow
        .immich_url
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("oauth flow expired; start again".into()))?;
    let base = Url::parse(raw_base).map_err(|_| AppError::Internal)?;
    let login = oauth::exchange_code(&base, &flow, &callback_url).await?;
    let user = oauth::login_user(&login);
    if !user.is_admin {
        return Err(AppError::AdminRequired);
    }
    let mut resp = auth::finish_setup(
        state,
        base.as_str(),
        &user,
        AuthKind::OAuth,
        login.access_token.as_bytes(),
        headers,
        client,
    )
    .await?;
    oauth::clear_flow_cookie(&mut resp);
    Ok(resp)
}
