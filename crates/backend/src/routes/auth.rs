use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::{AppError, REQUEST_ID};
use crate::immich::url::resolve_immich_base;
use crate::services::auth_store::AuthKind;
use crate::services::credentials::{validate_api_key, validate_password};
use crate::services::login_limiter::LoginKey;
use crate::services::login_session::{ClientMeta, cleared_session_cookie, finish_login, user_json};
use crate::state::AppState;

pub mod extract;
pub mod oauth;

pub use extract::{AdminCtx, AuthCtx};
use extract::{build_auth_ctx, extract_token, require_session};

#[derive(Deserialize)]
pub struct PasswordLoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ApiKeyLoginBody {
    pub api_key: String,
}

pub async fn login_password(
    State(state): State<AppState>,
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<PasswordLoginBody>,
) -> Result<Response, AppError> {
    let key = LoginKey::identity("login", &client.ip, &body.email);
    if let Some(d) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(d.as_secs())));
    }
    let base = resolve_immich_base(&state.instance).await?;
    let (user, cred) = match validate_password(&base, &body.email, &body.password).await {
        Ok(v) => v,
        Err(e) => {
            state.login_limiter.record_failure(&key);
            return Err(e);
        }
    };
    state.login_limiter.record_success(&key);
    finish_login(&state, &user, AuthKind::Password, &cred, &headers, &client).await
}

pub async fn login_api_key(
    State(state): State<AppState>,
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<ApiKeyLoginBody>,
) -> Result<Response, AppError> {
    let key = LoginKey::client("apikey", &client.ip);
    if let Some(d) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(d.as_secs())));
    }
    let base = resolve_immich_base(&state.instance).await?;
    let user = match validate_api_key(&base, &body.api_key).await {
        Ok(u) => u,
        Err(e) => {
            state.login_limiter.record_failure(&key);
            return Err(e);
        }
    };
    state.login_limiter.record_success(&key);
    finish_login(
        &state,
        &user,
        AuthKind::ApiKey,
        body.api_key.as_bytes(),
        &headers,
        &client,
    )
    .await
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    Ok((StatusCode::OK, Json(user_json(&ctx.user, ctx.auth_kind))).into_response())
}

pub async fn logout_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let request_id = REQUEST_ID.try_with(|s| s.clone()).unwrap_or_default();
    if let Ok(Some(ctx)) = build_auth_ctx(&state, &headers).await
        && ctx.auth_kind.revokes_upstream()
        && let Err(e) = ctx.immich.logout().await
    {
        tracing::warn!(
            %request_id,
            error = %e,
            "upstream logout failed; the immich token may still be valid"
        );
    }
    if let Some(token) = extract_token(&headers)
        && let Ok(Some(ctx)) = state.auth.authenticate(&token).await
    {
        if let Err(e) = state.jobs.cancel_active_for_session(ctx.session_id).await {
            tracing::warn!(%request_id, session = %ctx.session_id, error = %e, "job cancel on logout failed");
        }
        if let Err(e) = state.auth.revoke_session(ctx.session_id).await {
            tracing::warn!(%request_id, session = %ctx.session_id, error = %e, "session revoke on logout failed");
        }
    }
    let jar = CookieJar::new().add(cleared_session_cookie());
    (StatusCode::OK, jar, Json(json!({"ok": true}))).into_response()
}

pub async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    let sessions = state.auth.list_sessions(ctx.user.id).await?;
    let body: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "current": s.id == ctx.session_id,
                "created_at": s.created_at,
                "last_seen_at": s.last_seen_at,
                "user_agent": s.user_agent,
                "ip": s.ip,
            })
        })
        .collect();
    Ok((StatusCode::OK, Json(json!({ "sessions": body }))).into_response())
}

pub async fn revoke_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    let sessions = state.auth.list_sessions(ctx.user.id).await?;
    if !sessions.iter().any(|s| s.id == id) {
        return Err(AppError::NotFound);
    }
    state.jobs.cancel_active_for_session(id).await?;
    state.auth.revoke_session(id).await?;
    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}

pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    state
        .jobs
        .cancel_active_for_other_sessions(ctx.user.id, ctx.session_id)
        .await?;
    state
        .auth
        .revoke_others_for_user(ctx.user.id, ctx.session_id)
        .await?;
    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}
