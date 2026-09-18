mod common;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode};
use common::*;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROXY: &str = "127.0.0.1:9000";

async fn mock_oauth(server: &MockServer, admin: bool) {
    Mock::given(method("GET"))
        .and(path("/api/server/features"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oauth": true,
            "oauthAutoLaunch": false,
            "passwordLogin": true,
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/server/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oauthButtonText": "Sign in with Authentik",
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/oauth/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "url": "https://idp.example/auth" })),
        )
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/oauth/callback"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "accessToken": "oauth-token",
            "userId": test_user_id(),
            "userEmail": "admin@test.local",
            "name": "Admin",
            "isAdmin": admin,
        })))
        .mount(server)
        .await;
}

fn post(uri: &str, body: Value, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header("x-forwarded-for", "203.0.113.9")
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
        .find(|v| v.starts_with("immich_edit_oauth=") && !v.starts_with("immich_edit_oauth=;"))
        .map(|v| v.split(';').next().unwrap().to_string())
        .expect("no oauth flow cookie")
}

async fn sent_state(server: &MockServer) -> String {
    let requests = server.received_requests().await.unwrap();
    let body: Value = requests
        .iter()
        .rev()
        .find(|r| r.url.path() == "/api/oauth/authorize")
        .map(|r| serde_json::from_slice(&r.body).unwrap())
        .expect("no authorize call");
    body["state"].as_str().unwrap().to_string()
}

async fn start_setup(app: &axum::Router, server: &MockServer) -> (String, String) {
    let body = json!({
        "immich_url": server.uri(),
        "redirect_uri": "http://localhost/setup",
    });
    let resp = app
        .clone()
        .oneshot(post("/api/setup/oauth/start", body, None))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("start: expected 200, got {}", resp.status());
    }
    (flow_cookie(&resp), sent_state(server).await)
}

async fn complete_setup(app: &axum::Router, cookie: &str, state: &str) -> Response<Body> {
    let url = format!("http://localhost/setup?code=abc&state={state}");
    app.clone()
        .oneshot(post(
            "/api/setup/oauth/complete",
            json!({ "url": url }),
            Some(cookie),
        ))
        .await
        .unwrap()
}

async fn configured(app: &axum::Router) -> bool {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(resp).await["configured"] == json!(true)
}

#[tokio::test]
async fn setup_providers_describe_the_candidate_server() {
    let server = MockServer::start().await;
    mock_oauth(&server, true).await;
    let app = router(test_state(&server).await);

    let uri = format!("/api/setup/providers?immich_url={}", server.uri());
    let resp = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("expected 200, got {}", resp.status());
    }
    let body = body_json(resp).await;
    if body["oauth"] != json!(true) || body["button_text"] != json!("Sign in with Authentik") {
        panic!("unexpected providers: {body}");
    }
}

#[tokio::test]
async fn an_admin_can_claim_the_instance_over_oidc() {
    let server = MockServer::start().await;
    mock_oauth(&server, true).await;
    let app = router(test_state(&server).await);

    let (cookie, state) = start_setup(&app, &server).await;
    let resp = complete_setup(&app, &cookie, &state).await;
    if resp.status() != StatusCode::OK {
        panic!("complete: expected 200, got {}", resp.status());
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
    if body["auth_kind"] != json!("oauth") || body["is_admin"] != json!(true) {
        panic!("unexpected setup session: {body}");
    }
    if !configured(&app).await {
        panic!("the instance should be configured after setup");
    }
}

#[tokio::test]
async fn a_member_cannot_claim_the_instance_over_oidc() {
    let server = MockServer::start().await;
    mock_oauth(&server, false).await;
    let app = router(test_state(&server).await);

    let (cookie, state) = start_setup(&app, &server).await;
    let resp = complete_setup(&app, &cookie, &state).await;
    if resp.status() != StatusCode::FORBIDDEN {
        panic!("expected 403, got {}", resp.status());
    }
    if configured(&app).await {
        panic!("a rejected setup must leave the instance unconfigured");
    }
}

#[tokio::test]
async fn a_second_oidc_setup_is_a_conflict() {
    let server = MockServer::start().await;
    mock_oauth(&server, true).await;
    let app = router(test_state(&server).await);

    let (cookie, state) = start_setup(&app, &server).await;
    complete_setup(&app, &cookie, &state).await;

    let again = complete_setup(&app, &cookie, &state).await;
    if again.status() != StatusCode::CONFLICT {
        panic!("second complete: expected 409, got {}", again.status());
    }
    let start_again = app
        .clone()
        .oneshot(post(
            "/api/setup/oauth/start",
            json!({ "immich_url": server.uri(), "redirect_uri": "http://localhost/setup" }),
            None,
        ))
        .await
        .unwrap();
    if start_again.status() != StatusCode::CONFLICT {
        panic!("second start: expected 409, got {}", start_again.status());
    }
}

#[tokio::test]
async fn a_setup_flow_cannot_be_replayed_against_the_login_route() {
    let server = MockServer::start().await;
    mock_oauth(&server, true).await;
    let state_handle = test_state(&server).await;
    let app = router(state_handle.clone());

    let (cookie, flow_state) = start_setup(&app, &server).await;
    state_handle.instance.claim(&server.uri()).await.unwrap();

    let url = format!("http://localhost/setup?code=abc&state={flow_state}");
    let resp = app
        .oneshot(post(
            "/api/auth/oauth/callback",
            json!({ "url": url }),
            Some(&cookie),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("expected 400, got {}", resp.status());
    }
    let requests = server.received_requests().await.unwrap();
    if requests
        .iter()
        .any(|r| r.url.path() == "/api/oauth/callback")
    {
        panic!("a setup flow must not be exchanged by the login route");
    }
}
