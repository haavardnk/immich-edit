use axum::Json;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::header::{AUTHORIZATION, COOKIE, HeaderMap, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::{ImmichAuth, ImmichClient, ImmichUser};
use crate::services::auth_store::{AuthContext, AuthKind};
use crate::state::AppState;

pub const AUTH_COOKIE: &str = "immich_edit_auth";
pub const LEGACY_OWNER: Uuid = Uuid::nil();

#[derive(Clone)]
pub struct AuthCtx {
    pub owner: Uuid,
    pub server_epoch: i64,
    pub is_admin: bool,
    pub immich: ImmichClient,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthCtx {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, AppError> {
        parts
            .extensions
            .get::<AuthCtx>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

pub struct AdminCtx(pub AuthCtx);

impl<S: Send + Sync> FromRequestParts<S> for AdminCtx {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let ctx = AuthCtx::from_request_parts(parts, state).await?;
        if ctx.is_admin {
            Ok(AdminCtx(ctx))
        } else {
            Err(AppError::AdminRequired)
        }
    }
}

pub async fn build_auth_ctx(state: &AppState, headers: &HeaderMap) -> Option<AuthCtx> {
    if let Some(token) = extract_token(headers)
        && let Ok(Some(actx)) = state.auth.authenticate(&token).await
    {
        let base = resolve_immich_base(state).await.ok()?;
        let cred = actx.immich_cred.to_utf8()?;
        let auth = match actx.auth_kind {
            AuthKind::Password => ImmichAuth::Bearer(cred),
            AuthKind::ApiKey => ImmichAuth::ApiKey(cred),
        };
        let immich = ImmichClient::with_auth(
            base,
            auth,
            Duration::from_secs(state.config.original_timeout_secs),
        )
        .ok()?;
        return Some(AuthCtx {
            owner: actx.user.id,
            server_epoch: actx.server_epoch,
            is_admin: actx.user.is_admin,
            immich,
        });
    }
    let cfg = state.instance.get().await.ok()?;
    if !cfg.is_configured() {
        return Some(AuthCtx {
            owner: LEGACY_OWNER,
            server_epoch: 0,
            is_admin: true,
            immich: state.immich.clone(),
        });
    }
    None
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub token: String,
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginBody>) -> Response {
    let Some(expected) = state.config.auth_token.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"code":"auth_disabled","message":"auth not configured"})),
        )
            .into_response();
    };
    if !ct_eq(body.token.as_bytes(), expected.as_bytes()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"code":"unauthorized","message":"invalid token"})),
        )
            .into_response();
    }
    let cookie = format!(
        "{AUTH_COOKIE}={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000",
        body.token
    );
    let mut resp = (StatusCode::OK, Json(json!({"ok": true}))).into_response();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    resp
}

pub async fn logout() -> Response {
    let cookie = format!("{AUTH_COOKIE}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    let mut resp = (StatusCode::OK, Json(json!({"ok": true}))).into_response();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    resp
}

pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(rest) = auth.strip_prefix("Bearer ") {
            return Some(rest.to_string());
        }
    }
    let cookies = headers.get(COOKIE).and_then(|v| v.to_str().ok())?;
    for pair in cookies.split(';') {
        let trimmed = pair.trim();
        if let Some(rest) = trimmed
            .strip_prefix(AUTH_COOKIE)
            .and_then(|r| r.strip_prefix('='))
        {
            return Some(rest.to_string());
        }
    }
    None
}

pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[derive(Deserialize)]
pub struct PasswordLoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ApiKeyLoginBody {
    pub api_key: String,
}

pub async fn resolve_immich_base(state: &AppState) -> Result<Url, AppError> {
    if let Ok(cfg) = state.instance.get().await
        && let Some(url) = cfg.immich_url
    {
        return Url::parse(&url).map_err(|_| AppError::Internal);
    }
    Ok(state.config.immich_url.clone())
}

fn request_is_secure(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("https"))
        .unwrap_or(false)
}

fn session_cookie(token: &str, secure: bool) -> String {
    let base = format!("{AUTH_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000");
    if secure {
        format!("{base}; Secure")
    } else {
        base
    }
}

fn user_json(user: &crate::services::auth_store::UserRecord, kind: AuthKind) -> serde_json::Value {
    json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "is_admin": user.is_admin,
        "auth_kind": match kind {
            AuthKind::Password => "password",
            AuthKind::ApiKey => "apikey",
        },
    })
}

fn rate_key(ip: &str, ident: &str) -> String {
    format!("{ip}|{}", ident.to_lowercase())
}

pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

pub async fn require_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, AppError> {
    let token = extract_token(headers).ok_or(AppError::Unauthorized)?;
    match state.auth.authenticate(&token).await {
        Ok(Some(ctx)) => Ok(ctx),
        Ok(None) => Err(AppError::Unauthorized),
        Err(_) => Err(AppError::Internal),
    }
}

pub async fn finish_login(
    state: &AppState,
    user: &ImmichUser,
    kind: AuthKind,
    cred: &[u8],
    headers: &HeaderMap,
    ip: Option<&str>,
) -> Result<Response, AppError> {
    let epoch = state
        .instance
        .get()
        .await
        .map(|c| c.server_epoch)
        .unwrap_or(0);
    let stored = state
        .auth
        .upsert_user(user)
        .await
        .map_err(|_| AppError::Internal)?;
    if !stored.access_enabled {
        return Err(AppError::AccessDisabled);
    }
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(256).collect::<String>());
    let token = state
        .auth
        .create_session(stored.id, kind, cred, epoch, ua.as_deref(), ip)
        .await
        .map_err(|_| AppError::Internal)?;
    let cookie = session_cookie(&token, request_is_secure(headers));
    let mut resp = (StatusCode::OK, Json(user_json(&stored, kind))).into_response();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    Ok(resp)
}

pub async fn validate_password(
    base: &Url,
    email: &str,
    password: &str,
) -> Result<(ImmichUser, Vec<u8>), AppError> {
    let candidate = ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(String::new()),
        Duration::from_secs(30),
    )
    .map_err(|_| AppError::Internal)?;
    let login = candidate
        .login_password(email, password)
        .await
        .map_err(|_| AppError::Unauthorized)?;
    let user = ImmichUser {
        id: login.user_id,
        email: login.user_email,
        name: login.name,
        is_admin: login.is_admin,
    };
    Ok((user, login.access_token.into_bytes()))
}

pub async fn validate_api_key(base: &Url, api_key: &str) -> Result<ImmichUser, AppError> {
    let client = ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(api_key.to_string()),
        Duration::from_secs(30),
    )
    .map_err(|_| AppError::Internal)?;
    client.me().await.map_err(|_| AppError::Unauthorized)
}

pub async fn login_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasswordLoginBody>,
) -> Result<Response, AppError> {
    let ip = client_ip(&headers);
    let key = rate_key(&ip, &body.email);
    if let Some(d) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(d.as_secs())));
    }
    let base = resolve_immich_base(&state).await?;
    let (user, cred) = match validate_password(&base, &body.email, &body.password).await {
        Ok(v) => v,
        Err(e) => {
            state.login_limiter.record_failure(&key);
            return Err(e);
        }
    };
    state.login_limiter.record_success(&key);
    finish_login(
        &state,
        &user,
        AuthKind::Password,
        &cred,
        &headers,
        Some(ip.as_str()),
    )
    .await
}

pub async fn login_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ApiKeyLoginBody>,
) -> Result<Response, AppError> {
    let ip = client_ip(&headers);
    let key = rate_key(&ip, "apikey");
    if let Some(d) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(d.as_secs())));
    }
    let base = resolve_immich_base(&state).await?;
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
        Some(ip.as_str()),
    )
    .await
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    Ok((StatusCode::OK, Json(user_json(&ctx.user, ctx.auth_kind))).into_response())
}

pub async fn logout_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = extract_token(&headers)
        && let Ok(Some(ctx)) = state.auth.authenticate(&token).await
    {
        let _ = state.auth.revoke_session(ctx.session_id).await;
    }
    let cookie = format!("{AUTH_COOKIE}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    let mut resp = (StatusCode::OK, Json(json!({"ok": true}))).into_response();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    resp
}

pub async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    let sessions = state
        .auth
        .list_sessions(ctx.user.id)
        .await
        .map_err(|_| AppError::Internal)?;
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
    let sessions = state
        .auth
        .list_sessions(ctx.user.id)
        .await
        .map_err(|_| AppError::Internal)?;
    if !sessions.iter().any(|s| s.id == id) {
        return Err(AppError::NotFound);
    }
    state
        .auth
        .revoke_session(id)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}

pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    state
        .auth
        .revoke_all_for_user(ctx.user.id)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}
