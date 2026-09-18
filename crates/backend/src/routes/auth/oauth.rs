use axum::Json;
use axum::extract::State;
use axum::http::header::{COOKIE, HeaderMap, ORIGIN, SET_COOKIE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use url::Url;

use super::{ClientMeta, resolve_immich_base, start_session, user_json};
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::client::{ImmichAuth, ImmichClient, ImmichLogin, ImmichUser};
use crate::services::auth_store::AuthKind;
use crate::services::crypto::InstanceCrypto;
use crate::services::login_limiter::LoginKey;
use crate::services::oauth_flow::{self, FlowPurpose, OAuthFlow};
use crate::state::AppState;

const EXCHANGE_TIMEOUT: Duration = Duration::from_secs(30);
const CALLBACK_PATHS: [&str; 2] = ["/login", "/setup"];
// Resolving against an unreachable origin catches escapes the browser would make itself.
const NEXT_BASE: &str = "https://next.invalid";

#[derive(Deserialize)]
pub struct StartBody {
    pub redirect_uri: String,
    pub next: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackBody {
    pub url: String,
}

pub fn public_client(base: &Url) -> Result<ImmichClient, AppError> {
    ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(String::new()),
        EXCHANGE_TIMEOUT,
    )
    .map_err(AppError::from)
}

pub fn validate_redirect_uri(raw: &str, headers: &HeaderMap) -> Result<Url, AppError> {
    let url =
        Url::parse(raw.trim()).map_err(|_| AppError::BadRequest("invalid redirect uri".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest("redirect uri must be http(s)".into()));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AppError::BadRequest(
            "redirect uri must not carry a query or fragment".into(),
        ));
    }
    if !CALLBACK_PATHS.contains(&url.path()) {
        return Err(AppError::BadRequest(
            "redirect uri path is not a callback".into(),
        ));
    }
    let origin = headers
        .get(ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("origin header required".into()))?;
    if url.origin().ascii_serialization() != origin {
        return Err(AppError::BadRequest(
            "redirect uri must match the request origin".into(),
        ));
    }
    Ok(url)
}

pub fn safe_next(next: Option<String>) -> Option<String> {
    let base = Url::parse(NEXT_BASE).ok()?;
    let url = base.join(&next?).ok()?;
    if url.origin() != base.origin() {
        return None;
    }
    let mut target = url.path().to_string();
    if let Some(query) = url.query() {
        target.push('?');
        target.push_str(query);
    }
    if let Some(fragment) = url.fragment() {
        target.push('#');
        target.push_str(fragment);
    }
    Some(target)
}

pub async fn begin_flow(
    base: &Url,
    flow: &OAuthFlow,
    crypto: &InstanceCrypto,
    secure: bool,
) -> Result<Response, AppError> {
    let url = public_client(base)?
        .oauth_authorize(&flow.redirect_uri, &flow.state, &flow.challenge())
        .await?;
    let sealed = flow.seal(crypto)?;
    let mut resp = (StatusCode::OK, Json(json!({ "url": url }))).into_response();
    if let Ok(v) = HeaderValue::from_str(&oauth_flow::set_cookie(&sealed, secure)) {
        resp.headers_mut().append(SET_COOKIE, v);
    }
    Ok(resp)
}

pub fn open_flow(
    state: &AppState,
    headers: &HeaderMap,
    purpose: FlowPurpose,
) -> Result<OAuthFlow, AppError> {
    let raw = headers.get(COOKIE).and_then(|v| v.to_str().ok());
    let sealed = oauth_flow::read_cookie(raw)
        .ok_or_else(|| AppError::BadRequest("oauth flow expired; start again".into()))?;
    let flow = OAuthFlow::open(&state.crypto, &sealed)?;
    if flow.purpose != purpose {
        return Err(AppError::BadRequest(
            "oauth flow expired; start again".into(),
        ));
    }
    Ok(flow)
}

pub fn validate_callback_url(raw: &str, flow: &OAuthFlow) -> Result<Url, AppError> {
    let url =
        Url::parse(raw.trim()).map_err(|_| AppError::BadRequest("invalid callback url".into()))?;
    let expected = Url::parse(&flow.redirect_uri).map_err(|_| AppError::Internal)?;
    if url.origin() != expected.origin() || url.path() != expected.path() {
        return Err(AppError::BadRequest(
            "callback url does not match the started flow".into(),
        ));
    }
    let state_matches = url
        .query_pairs()
        .any(|(k, v)| k == "state" && v == flow.state.as_str());
    if !state_matches {
        return Err(AppError::BadRequest(
            "callback url does not match the started flow".into(),
        ));
    }
    Ok(url)
}

pub async fn exchange_code(
    base: &Url,
    flow: &OAuthFlow,
    callback_url: &Url,
) -> Result<ImmichLogin, AppError> {
    public_client(base)?
        .oauth_callback(callback_url.as_str(), &flow.state, &flow.verifier)
        .await
        .map_err(map_exchange_error)
}

fn map_exchange_error(err: ImmichError) -> AppError {
    match err {
        ImmichError::Unauthorized | ImmichError::Status(400) => AppError::Unauthorized,
        other => other.into(),
    }
}

pub fn login_user(login: &ImmichLogin) -> ImmichUser {
    ImmichUser {
        id: login.user_id,
        email: login.user_email.clone(),
        name: login.name.clone(),
        is_admin: login.is_admin,
    }
}

pub async fn providers(State(state): State<AppState>) -> Result<Response, AppError> {
    let base = resolve_immich_base(&state).await?;
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
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<StartBody>,
) -> Result<Response, AppError> {
    let base = resolve_immich_base(&state).await?;
    let redirect = validate_redirect_uri(&body.redirect_uri, &headers)?;
    let flow = OAuthFlow::begin(FlowPurpose::Login, &redirect, safe_next(body.next), None);
    begin_flow(&base, &flow, &state.crypto, client.secure).await
}

pub async fn callback(
    State(state): State<AppState>,
    client: ClientMeta,
    headers: HeaderMap,
    Json(body): Json<CallbackBody>,
) -> Result<Response, AppError> {
    let key = LoginKey::client("oauth", &client.ip);
    if let Some(d) = state.login_limiter.retry_after(&key) {
        return Err(AppError::RateLimited(Some(d.as_secs())));
    }
    match finish_callback(&state, &client, &headers, &body).await {
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

async fn finish_callback(
    state: &AppState,
    client: &ClientMeta,
    headers: &HeaderMap,
    body: &CallbackBody,
) -> Result<Response, AppError> {
    let flow = open_flow(state, headers, FlowPurpose::Login)?;
    let callback_url = validate_callback_url(&body.url, &flow)?;
    let base = resolve_immich_base(state).await?;
    let login = exchange_code(&base, &flow, &callback_url).await?;
    let user = login_user(&login);
    let (stored, token) = start_session(
        state,
        &user,
        AuthKind::OAuth,
        login.access_token.as_bytes(),
        headers,
        client,
    )
    .await?;
    let mut payload = user_json(&stored, AuthKind::OAuth);
    if let Some(next) = flow.next.clone()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("next".into(), json!(next));
    }
    let mut resp = (StatusCode::OK, Json(payload)).into_response();
    let cookies = [
        super::session_cookie(&token, client.secure),
        oauth_flow::clear_cookie(),
    ];
    for cookie in cookies {
        if let Ok(v) = HeaderValue::from_str(&cookie) {
            resp.headers_mut().append(SET_COOKIE, v);
        }
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::safe_next;

    #[test]
    fn a_next_target_that_leaves_the_origin_is_dropped() {
        for raw in [
            "//evil.test",
            "/\\evil.test",
            "/\t/evil.test",
            "/\n/evil.test",
            "/\r/evil.test",
            "https://evil.test",
            "javascript:alert(1)",
        ] {
            assert_eq!(safe_next(Some(raw.into())), None, "{raw:?}");
        }
    }

    #[test]
    fn a_same_origin_next_target_keeps_its_query() {
        assert_eq!(
            safe_next(Some("/albums?sort=taken".into())),
            Some("/albums?sort=taken".into())
        );
        assert_eq!(safe_next(None), None);
    }
}
