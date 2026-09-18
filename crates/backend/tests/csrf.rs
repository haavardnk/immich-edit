mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use tower::ServiceExt;
use wiremock::MockServer;

async fn app_and_token(server: &MockServer) -> (axum::Router, String) {
    let state = test_state(server).await;
    let token = seed_session(server, &state).await;
    (router(state), token)
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    host: &str,
    origin: Option<&str>,
    cookie: Option<&str>,
) -> StatusCode {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", host);
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    if let Some(token) = cookie {
        builder = builder.header("cookie", format!("immich_edit_auth={token}"));
    }
    let req = builder.body(Body::empty()).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn a_cookie_post_without_origin_is_forbidden() {
    let server = MockServer::start().await;
    let (app, token) = app_and_token(&server).await;
    let status = send(
        &app,
        "POST",
        "/api/auth/logout",
        "localhost",
        None,
        Some(&token),
    )
    .await;
    if status != StatusCode::FORBIDDEN {
        panic!("an origin-less cookie post must be rejected, got {status}");
    }
}

#[tokio::test]
async fn a_matching_origin_passes_the_guard() {
    let server = MockServer::start().await;
    let (app, token) = app_and_token(&server).await;
    let status = send(
        &app,
        "POST",
        "/api/auth/logout",
        "localhost",
        Some("http://localhost"),
        Some(&token),
    )
    .await;
    if status != StatusCode::OK {
        panic!("a same-origin post must pass, got {status}");
    }
}

#[tokio::test]
async fn a_foreign_origin_is_forbidden() {
    let server = MockServer::start().await;
    let (app, token) = app_and_token(&server).await;
    let status = send(
        &app,
        "POST",
        "/api/auth/logout",
        "localhost",
        Some("https://evil.example"),
        Some(&token),
    )
    .await;
    if status != StatusCode::FORBIDDEN {
        panic!("a cross-origin post must be rejected, got {status}");
    }
}

#[tokio::test]
async fn the_origin_port_must_match_the_host() {
    let server = MockServer::start().await;
    let (app, token) = app_and_token(&server).await;
    let mismatched = send(
        &app,
        "POST",
        "/api/auth/logout",
        "localhost:5173",
        Some("http://localhost:3000"),
        Some(&token),
    )
    .await;
    if mismatched != StatusCode::FORBIDDEN {
        panic!("a port mismatch must be rejected, got {mismatched}");
    }
    let matched = send(
        &app,
        "POST",
        "/api/auth/logout",
        "localhost:5173",
        Some("http://localhost:5173"),
        Some(&token),
    )
    .await;
    if matched != StatusCode::OK {
        panic!("an exact host:port match must pass, got {matched}");
    }
}

#[tokio::test]
async fn a_cookieless_post_without_origin_passes() {
    let server = MockServer::start().await;
    let (app, _token) = app_and_token(&server).await;
    let status = send(&app, "POST", "/api/auth/logout", "localhost", None, None).await;
    if status != StatusCode::OK {
        panic!("a cookieless post must reach the route, got {status}");
    }
}

#[tokio::test]
async fn safe_methods_skip_the_guard() {
    let server = MockServer::start().await;
    let (app, token) = app_and_token(&server).await;
    let status = send(
        &app,
        "GET",
        "/api/auth/me",
        "localhost",
        Some("https://evil.example"),
        Some(&token),
    )
    .await;
    if status != StatusCode::OK {
        panic!("a cross-origin GET must still be served, got {status}");
    }
}
