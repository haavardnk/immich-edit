mod common;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::*;
use immich_edit_backend::immich::client::ImmichUser;
use immich_edit_backend::services::auth_store::AuthKind;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROXY: &str = "127.0.0.1:9000";
const IDP_URL: &str = "https://idp.example/authorize?client_id=immich";

async fn configured_app(server: &MockServer) -> axum::Router {
    let state = test_state(server).await;
    state.instance.claim(&server.uri()).await.unwrap();
    router(state)
}

async fn mock_features(server: &MockServer, oauth: bool, auto_launch: bool, password: bool) {
    Mock::given(method("GET"))
        .and(path("/api/server/features"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oauth": oauth,
            "oauthAutoLaunch": auto_launch,
            "passwordLogin": password,
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/server/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oauthButtonText": "Sign in with Keycloak",
        })))
        .mount(server)
        .await;
}

async fn mock_authorize(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/oauth/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "url": IDP_URL })))
        .mount(server)
        .await;
}

async fn mock_callback(server: &MockServer, status: u16) {
    let response = if status < 400 {
        ResponseTemplate::new(status).set_body_json(json!({
            "accessToken": "oauth-token",
            "userId": test_user_id(),
            "userEmail": "admin@test.local",
            "name": "Admin",
            "isAdmin": true,
        }))
    } else {
        ResponseTemplate::new(status)
    };
    Mock::given(method("POST"))
        .and(path("/api/oauth/callback"))
        .respond_with(response)
        .mount(server)
        .await;
}

fn post(uri: &str, body: Value, cookie: Option<&str>, ip: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header("x-forwarded-for", ip)
        .extension(ConnectInfo::<SocketAddr>(PROXY.parse().unwrap()));
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn body_json(resp: Response<Body>) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn flow_cookie(resp: &Response<Body>) -> String {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("immich_edit_oauth="))
        .map(|v| v.split(';').next().unwrap().to_string())
        .expect("no oauth flow cookie")
}

async fn sent_body(server: &MockServer, route: &str) -> Option<Value> {
    let requests = server.received_requests().await.unwrap();
    requests
        .iter()
        .rev()
        .find(|r| r.url.path() == route)
        .map(|r| serde_json::from_slice(&r.body).unwrap())
}

async fn start_flow(app: &axum::Router, server: &MockServer) -> (String, Value) {
    let body = json!({ "redirect_uri": "http://localhost/login", "next": "/albums/42" });
    let resp = app
        .clone()
        .oneshot(post("/api/auth/oauth/start", body, None, "203.0.113.1"))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("start: expected 200, got {}", resp.status());
    }
    let cookie = flow_cookie(&resp);
    let sent = sent_body(server, "/api/oauth/authorize").await.unwrap();
    (cookie, sent)
}

#[tokio::test]
async fn providers_report_what_the_immich_server_offers() {
    let server = MockServer::start().await;
    mock_features(&server, true, true, false).await;
    let app = configured_app(&server).await;

    let req = Request::builder()
        .uri("/api/auth/providers")
        .header("host", "localhost")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("expected 200, got {}", resp.status());
    }
    let body = body_json(resp).await;
    if body
        != json!({
            "oauth": true,
            "password_login": false,
            "auto_launch": true,
            "button_text": "Sign in with Keycloak",
            "degraded": false,
        })
    {
        panic!("unexpected providers: {body}");
    }
}

#[tokio::test]
async fn providers_fall_back_to_passwords_when_immich_is_unreachable() {
    let server = MockServer::start().await;
    let app = configured_app(&server).await;

    let req = Request::builder()
        .uri("/api/auth/providers")
        .header("host", "localhost")
        .body(Body::empty())
        .unwrap();
    let body = body_json(app.oneshot(req).await.unwrap()).await;
    if body["oauth"] != json!(false) || body["password_login"] != json!(true) {
        panic!("unexpected fallback: {body}");
    }
    if body["degraded"] != json!(true) {
        panic!("a failed lookup must be reported as degraded: {body}");
    }
}

#[tokio::test]
async fn start_returns_the_idp_url_and_a_sealed_flow_cookie() {
    let server = MockServer::start().await;
    mock_authorize(&server).await;
    let app = configured_app(&server).await;

    let body = json!({ "redirect_uri": "http://localhost/login" });
    let resp = app
        .oneshot(post("/api/auth/oauth/start", body, None, "203.0.113.1"))
        .await
        .unwrap();
    let cookie = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("immich_edit_oauth="))
        .unwrap()
        .to_string();
    if !cookie.contains("HttpOnly") || !cookie.contains("SameSite=Lax") {
        panic!("flow cookie must be HttpOnly and Lax: {cookie}");
    }
    let payload = body_json(resp).await;
    if payload["url"] != json!(IDP_URL) {
        panic!("expected the idp url, got {payload}");
    }
    let sent = sent_body(&server, "/api/oauth/authorize").await.unwrap();
    if sent["redirectUri"] != json!("http://localhost/login") {
        panic!("redirect uri was not forwarded: {sent}");
    }
    if sent["state"].as_str().unwrap_or_default().is_empty()
        || sent["codeChallenge"]
            .as_str()
            .unwrap_or_default()
            .is_empty()
    {
        panic!("state and pkce challenge must be forwarded: {sent}");
    }
}

#[tokio::test]
async fn the_callback_signs_the_user_in_with_the_matching_verifier() {
    let server = MockServer::start().await;
    mock_authorize(&server).await;
    mock_callback(&server, 201).await;
    let app = configured_app(&server).await;

    let (cookie, authorized) = start_flow(&app, &server).await;
    let state = authorized["state"].as_str().unwrap();
    let callback_url = format!("http://localhost/login?code=abc&state={state}");
    let resp = app
        .clone()
        .oneshot(post(
            "/api/auth/oauth/callback",
            json!({ "url": callback_url }),
            Some(&cookie),
            "203.0.113.1",
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("callback: expected 200, got {}", resp.status());
    }
    let cookies: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    if !cookies.iter().any(|c| c.starts_with("immich_edit_auth=")) {
        panic!("no session cookie: {cookies:?}");
    }
    if !cookies.iter().any(|c| c.starts_with("immich_edit_oauth=;")) {
        panic!("the flow cookie must be cleared: {cookies:?}");
    }
    let body = body_json(resp).await;
    if body["auth_kind"] != json!("oauth") {
        panic!("expected an oauth session: {body}");
    }
    if body["next"] != json!("/albums/42") {
        panic!("the sealed next target must survive the round trip: {body}");
    }

    let exchanged = sent_body(&server, "/api/oauth/callback").await.unwrap();
    if exchanged["url"] != json!(callback_url) || exchanged["state"] != json!(state) {
        panic!("callback was not forwarded verbatim: {exchanged}");
    }
    let verifier = exchanged["codeVerifier"].as_str().unwrap();
    let derived = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    if derived != authorized["codeChallenge"].as_str().unwrap() {
        panic!("the verifier does not hash to the challenge sent to immich");
    }
}

#[tokio::test]
async fn a_callback_with_a_foreign_state_never_reaches_immich() {
    let server = MockServer::start().await;
    mock_authorize(&server).await;
    mock_callback(&server, 201).await;
    let app = configured_app(&server).await;

    let (cookie, _) = start_flow(&app, &server).await;
    let resp = app
        .clone()
        .oneshot(post(
            "/api/auth/oauth/callback",
            json!({ "url": "http://localhost/login?code=abc&state=attacker" }),
            Some(&cookie),
            "203.0.113.1",
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("expected 400, got {}", resp.status());
    }
    if sent_body(&server, "/api/oauth/callback").await.is_some() {
        panic!("a forged state must not be exchanged upstream");
    }
}

#[tokio::test]
async fn a_callback_without_the_flow_cookie_is_rejected() {
    let server = MockServer::start().await;
    mock_callback(&server, 201).await;
    let app = configured_app(&server).await;

    let resp = app
        .oneshot(post(
            "/api/auth/oauth/callback",
            json!({ "url": "http://localhost/login?code=abc&state=whatever" }),
            None,
            "203.0.113.2",
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("expected 400, got {}", resp.status());
    }
    if sent_body(&server, "/api/oauth/callback").await.is_some() {
        panic!("a cookieless callback must not be exchanged upstream");
    }
}

#[tokio::test]
async fn a_redirect_uri_outside_the_request_origin_is_rejected() {
    let server = MockServer::start().await;
    mock_authorize(&server).await;
    let app = configured_app(&server).await;

    for candidate in [
        "https://evil.example/login",
        "http://localhost/albums",
        "http://localhost/login?next=%2Fevil",
        "javascript:alert(1)",
    ] {
        let resp = app
            .clone()
            .oneshot(post(
                "/api/auth/oauth/start",
                json!({ "redirect_uri": candidate }),
                None,
                "203.0.113.3",
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::BAD_REQUEST {
            panic!("{candidate}: expected 400, got {}", resp.status());
        }
    }
    if sent_body(&server, "/api/oauth/authorize").await.is_some() {
        panic!("a rejected redirect uri must not start a flow upstream");
    }
}

#[tokio::test]
async fn an_upstream_rejection_is_unauthorized_and_rate_limited() {
    let server = MockServer::start().await;
    mock_authorize(&server).await;
    mock_callback(&server, 400).await;
    let app = configured_app(&server).await;

    for n in 1..=5 {
        let (cookie, authorized) = start_flow(&app, &server).await;
        let state = authorized["state"].as_str().unwrap();
        let url = format!("http://localhost/login?code=abc&state={state}");
        let got = app
            .clone()
            .oneshot(post(
                "/api/auth/oauth/callback",
                json!({ "url": url }),
                Some(&cookie),
                "203.0.113.4",
            ))
            .await
            .unwrap()
            .status();
        if got != StatusCode::UNAUTHORIZED {
            panic!("attempt {n}: expected 401, got {got}");
        }
    }

    let (cookie, authorized) = start_flow(&app, &server).await;
    let state = authorized["state"].as_str().unwrap();
    let url = format!("http://localhost/login?code=abc&state={state}");
    let got = app
        .oneshot(post(
            "/api/auth/oauth/callback",
            json!({ "url": url }),
            Some(&cookie),
            "203.0.113.4",
        ))
        .await
        .unwrap()
        .status();
    if got != StatusCode::TOO_MANY_REQUESTS {
        panic!("sixth attempt: expected 429, got {got}");
    }
}

#[tokio::test]
async fn logging_out_of_an_oauth_session_revokes_the_immich_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/logout"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "successful": true })))
        .mount(&server)
        .await;
    let state = test_state(&server).await;
    let user = ImmichUser {
        id: test_user_id(),
        email: "admin@test.local".into(),
        name: "Admin".into(),
        is_admin: true,
    };
    let token =
        seed_session_with_cred(&server, &state, user, AuthKind::OAuth, b"oauth-token").await;
    let app = wrap_auth(router(state), token);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("logout: expected 200, got {}", resp.status());
    }
    let requests = server.received_requests().await.unwrap();
    let logged_out = requests
        .iter()
        .find(|r| r.url.path() == "/api/auth/logout")
        .expect("upstream logout was never called");
    let auth = logged_out
        .headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if auth != "Bearer oauth-token" {
        panic!("expected a bearer logout, got {auth:?}");
    }
}
