use axum::Json;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::header::{AUTHORIZATION, COOKIE, HeaderMap, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::{ImmichAuth, ImmichClient, ImmichUser};
use crate::services::auth_store::{AuthContext, AuthKind, UserRecord};
use crate::services::crypto::SecretBytes;
use crate::state::AppState;

pub const AUTH_COOKIE: &str = "immich_edit_auth";

#[derive(Clone)]
pub struct AuthCtx {
    pub owner: Uuid,
    pub session_id: Uuid,
    pub server_epoch: i64,
    pub is_admin: bool,
    pub immich: ImmichClient,
    pub cred: Arc<SecretBytes>,
    pub auth_kind: AuthKind,
}

impl From<&AuthCtx> for crate::services::render::RenderIdentity {
    fn from(ctx: &AuthCtx) -> Self {
        Self {
            owner: ctx.owner,
            server_epoch: ctx.server_epoch,
        }
    }
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
    let token = extract_token(headers)?;
    let actx = state.auth.authenticate(&token).await.ok()??;
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
    Some(AuthCtx {
        owner: actx.user.id,
        session_id: actx.session_id,
        server_epoch: actx.server_epoch,
        is_admin: actx.user.is_admin,
        immich,
        cred: Arc::new(actx.immich_cred),
        auth_kind: actx.auth_kind,
    })
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
    let cfg = state.instance.get().await?;
    let url = cfg.immich_url.ok_or(AppError::SetupRequired)?;
    Url::parse(&url).map_err(|_| AppError::Internal)
}

pub fn validate_candidate_url(raw: &str) -> Result<Url, AppError> {
    let url =
        Url::parse(raw.trim()).map_err(|_| AppError::BadRequest("invalid immich url".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest("immich url must be http(s)".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::BadRequest(
            "immich url must not contain credentials".into(),
        ));
    }
    let host = url
        .host()
        .ok_or_else(|| AppError::BadRequest("immich url must include a host".into()))?;
    let blocked = match host {
        url::Host::Ipv4(ip) => {
            ip.is_link_local() || ip.is_multicast() || ip.is_unspecified() || ip.is_broadcast()
        }
        url::Host::Ipv6(ip) => {
            ip.is_multicast() || ip.is_unspecified() || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
        url::Host::Domain(_) => false,
    };
    if blocked {
        return Err(AppError::BadRequest(
            "immich url host is not allowed".into(),
        ));
    }
    Ok(url)
}

#[derive(Clone)]
pub struct ClientMeta {
    pub ip: String,
    pub secure: bool,
}

impl Default for ClientMeta {
    fn default() -> Self {
        Self {
            ip: "unknown".into(),
            secure: false,
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for ClientMeta {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<ClientMeta>()
            .cloned()
            .unwrap_or_default())
    }
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

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(256).collect::<String>())
}

fn login_response(user: &UserRecord, kind: AuthKind, token: &str, secure: bool) -> Response {
    let cookie = session_cookie(token, secure);
    let mut response = (StatusCode::OK, Json(user_json(user, kind))).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(SET_COOKIE, value);
    }
    response
}

fn rate_key(ip: &str, ident: &str) -> String {
    format!("{ip}|{}", ident.to_lowercase())
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
    client: &ClientMeta,
) -> Result<Response, AppError> {
    let epoch = state
        .instance
        .get()
        .await
        .map(|c| c.server_epoch)
        .unwrap_or(0);
    let stored = state.auth.upsert_user(user).await?;
    if !stored.access_enabled {
        return Err(AppError::AccessDisabled);
    }
    let ua = user_agent(headers);
    let token = state
        .auth
        .create_session(
            stored.id,
            kind,
            cred,
            epoch,
            ua.as_deref(),
            Some(&client.ip),
        )
        .await?;
    Ok(login_response(&stored, kind, &token, client.secure))
}

pub async fn finish_setup(
    state: &AppState,
    immich_url: &str,
    user: &ImmichUser,
    kind: AuthKind,
    cred: &[u8],
    headers: &HeaderMap,
    client: &ClientMeta,
) -> Result<Response, AppError> {
    let ua = user_agent(headers);
    let (stored, token) = state
        .auth
        .claim_instance_and_create_session(
            immich_url,
            user,
            kind,
            cred,
            ua.as_deref(),
            Some(&client.ip),
        )
        .await?;
    Ok(login_response(&stored, kind, &token, client.secure))
}

pub async fn finish_rebind(
    state: &AppState,
    immich_url: &str,
    user: &ImmichUser,
    kind: AuthKind,
    cred: &[u8],
    headers: &HeaderMap,
    client: &ClientMeta,
) -> Result<Response, AppError> {
    let ua = user_agent(headers);
    let (stored, token) = state
        .auth
        .rebind_instance_and_create_session(
            immich_url,
            user,
            kind,
            cred,
            ua.as_deref(),
            Some(&client.ip),
        )
        .await?;
    Ok(login_response(&stored, kind, &token, client.secure))
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
    )?;
    let login = candidate
        .login_password(email, password)
        .await
        .map_err(map_login_error)?;
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
    )?;
    client.me().await.map_err(map_login_error)
}

pub async fn validate_credentials(
    base: &Url,
    email: Option<&str>,
    password: Option<&str>,
    api_key: Option<&str>,
) -> Result<(ImmichUser, AuthKind, Vec<u8>), AppError> {
    if let Some(api_key) = api_key {
        let user = validate_api_key(base, api_key).await?;
        return Ok((user, AuthKind::ApiKey, api_key.as_bytes().to_vec()));
    }
    if let (Some(email), Some(password)) = (email, password) {
        let (user, cred) = validate_password(base, email, password).await?;
        return Ok((user, AuthKind::Password, cred));
    }
    Err(AppError::BadRequest(
        "email+password or api_key required".into(),
    ))
}

fn map_login_error(err: crate::immich::ImmichError) -> AppError {
    match err {
        crate::immich::ImmichError::Unauthorized => AppError::Unauthorized,
        other => other.into(),
    }
}

pub async fn login_password(
    State(state): State<AppState>,
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<PasswordLoginBody>,
) -> Result<Response, AppError> {
    let key = rate_key(&client.ip, &body.email);
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
    finish_login(&state, &user, AuthKind::Password, &cred, &headers, &client).await
}

pub async fn login_api_key(
    State(state): State<AppState>,
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<ApiKeyLoginBody>,
) -> Result<Response, AppError> {
    let key = rate_key(&client.ip, "apikey");
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
        &client,
    )
    .await
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    let ctx = require_session(&state, &headers).await?;
    Ok((StatusCode::OK, Json(user_json(&ctx.user, ctx.auth_kind))).into_response())
}

pub async fn logout_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(ctx) = build_auth_ctx(&state, &headers).await
        && matches!(ctx.auth_kind, AuthKind::Password)
    {
        let _ = ctx.immich.logout().await;
    }
    if let Some(token) = extract_token(&headers)
        && let Ok(Some(ctx)) = state.auth.authenticate(&token).await
    {
        let _ = state.jobs.cancel_active_for_session(ctx.session_id).await;
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

#[cfg(test)]
mod tests {
    use super::validate_candidate_url;

    #[test]
    fn allows_loopback_and_private_hosts() {
        assert!(validate_candidate_url("http://127.0.0.1:2283").is_ok());
        assert!(validate_candidate_url("http://192.168.1.10:2283").is_ok());
        assert!(validate_candidate_url("https://immich.example.com").is_ok());
    }

    #[test]
    fn blocks_cloud_metadata_and_bad_schemes() {
        assert!(validate_candidate_url("http://169.254.169.254/").is_err());
        assert!(validate_candidate_url("http://0.0.0.0/").is_err());
        assert!(validate_candidate_url("ftp://immich.example.com").is_err());
        assert!(validate_candidate_url("http://user:pass@immich.example.com").is_err());
        assert!(validate_candidate_url("not a url").is_err());
    }
}
