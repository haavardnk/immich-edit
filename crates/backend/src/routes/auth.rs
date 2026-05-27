use axum::Json;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, COOKIE, HeaderMap, SET_COOKIE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

pub const AUTH_COOKIE: &str = "immich_edit_auth";

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
