use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tracing::Level;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_failed_upstream_logout_is_logged() {
    let (capture, _guard) = capture_logs(Level::WARN);

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/logout"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let app = password_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if res.status() != StatusCode::OK {
        panic!(
            "logout must still clear the local session, got {}",
            res.status()
        );
    }
    let logged = capture.text();
    if !logged.contains("upstream logout failed") {
        panic!("a failed upstream logout must be logged, got {logged:?}");
    }
}

#[tokio::test]
async fn logout_revokes_upstream_password_session_only() {
    for (password, calls) in [(true, 1), (false, 0)] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/auth/logout"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(calls)
            .mount(&server)
            .await;
        let app = if password {
            password_app(&server).await
        } else {
            test_app(&server).await
        };
        let resp = app
            .oneshot(empty_request("POST", "/api/auth/logout"))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("logout status {} (password {password})", resp.status());
        }
        server.verify().await;
    }
}
