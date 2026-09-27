use axum::Json;
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::json;
use time::Duration;

use crate::error::AppError;
use crate::immich::client::ImmichUser;
use crate::services::auth_store::{AuthKind, UserRecord};
use crate::state::AppState;

pub const AUTH_COOKIE: &str = "immich_edit_auth";
const SESSION_MAX_AGE: Duration = Duration::days(30);

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

pub fn session_cookie(token: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE, token.to_string()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(SESSION_MAX_AGE)
        .secure(secure)
        .build()
}

pub fn cleared_session_cookie() -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(Duration::ZERO)
        .build()
}

pub fn user_json(user: &UserRecord, kind: AuthKind) -> serde_json::Value {
    json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "is_admin": user.is_admin,
        "auth_kind": kind.as_str(),
    })
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(256).collect::<String>())
}

fn login_response(user: &UserRecord, kind: AuthKind, token: &str, secure: bool) -> Response {
    let jar = CookieJar::new().add(session_cookie(token, secure));
    (StatusCode::OK, jar, Json(user_json(user, kind))).into_response()
}

pub async fn start_session(
    state: &AppState,
    user: &ImmichUser,
    kind: AuthKind,
    cred: &[u8],
    headers: &HeaderMap,
    client: &ClientMeta,
) -> Result<(UserRecord, String), AppError> {
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
    Ok((stored, token))
}

pub async fn finish_login(
    state: &AppState,
    user: &ImmichUser,
    kind: AuthKind,
    cred: &[u8],
    headers: &HeaderMap,
    client: &ClientMeta,
) -> Result<Response, AppError> {
    let (stored, token) = start_session(state, user, kind, cred, headers, client).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_cookie_is_strict_and_secure_only_behind_tls() {
        assert_eq!(
            session_cookie("tok", true).to_string(),
            "immich_edit_auth=tok; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=2592000"
        );
        assert_eq!(
            session_cookie("tok", false).to_string(),
            "immich_edit_auth=tok; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000"
        );
    }

    #[test]
    fn the_cleared_cookie_expires_the_session_path() {
        assert_eq!(
            cleared_session_cookie().to_string(),
            "immich_edit_auth=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0"
        );
    }
}
