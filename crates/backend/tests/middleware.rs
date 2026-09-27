mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn unknown_api_returns_json_404() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/does/not/exist")).await.unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = header_str(&resp, "content-type").unwrap_or_default();
    if !ct.contains("application/json") {
        panic!("content-type: {ct}");
    }
    let json = body_json(resp).await;
    if json["code"] != "not_found" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn unknown_non_api_returns_plain_404() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/something")).await.unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = header_str(&resp, "content-type").unwrap_or_default();
    if ct.contains("application/json") {
        panic!("non-api should not be JSON 404: {ct}");
    }
}

#[tokio::test]
async fn protected_route_requires_session_when_configured() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    state.instance.claim(&server.uri()).await.unwrap();
    let app = router(state);
    let resp = app.oneshot(get("/api/health")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_accepts_bearer_token() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn protected_route_rejects_invalid_session_when_configured() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    state.instance.claim(&server.uri()).await.unwrap();
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("cookie", "immich_edit_auth=bogus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn request_id_header_propagated() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app.oneshot(get("/api/health/live")).await.unwrap();
    assert!(resp.headers().get("x-request-id").is_some());
}

#[tokio::test]
async fn error_body_request_id_matches_inbound_header() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/does/not/exist")
                .header("x-request-id", "client-supplied-1234")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        header_str(&resp, "x-request-id").as_deref(),
        Some("client-supplied-1234")
    );
    let json = body_json(resp).await;
    assert_eq!(json["request_id"], "client-supplied-1234");
}
