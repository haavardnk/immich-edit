use crate::common::*;
use serde_json::json;
use tower::ServiceExt;
use tracing::Level;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_rejected_request_logs_one_line_without_the_query() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "message": "nope" })))
        .mount(&server)
        .await;

    let resp = test_app(&server)
        .await
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{}?token=hunter2", asset_id()),
            json!({"isFavorite": true}),
        ))
        .await
        .unwrap();

    if resp.status() != axum::http::StatusCode::UNPROCESSABLE_ENTITY {
        panic!("status {}", resp.status());
    }
    let lines = capture.lines_with("request rejected");
    let [line] = lines.as_slice() else {
        panic!("expected one access line, got {lines:?}");
    };
    for needle in [
        "request_id=",
        &format!("path=\"/api/assets/{}\"", asset_id()),
        "status=422",
        "upstream_rejected",
        "nope",
    ] {
        if !line.contains(needle) {
            panic!("access line misses {needle:?}: {line}");
        }
    }
    if capture.text().contains("hunter2") {
        panic!("query strings must never be logged");
    }
}

#[tokio::test]
async fn a_malformed_body_logs_the_reason() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let req = axum::http::Request::builder()
        .method("PUT")
        .uri(format!("/api/assets/{}", asset_id()))
        .header("content-type", "application/json")
        .body(axum::body::Body::from("{"))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    if resp.status() != axum::http::StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
    let reason = "Failed to parse the request body as JSON";
    let body = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    if !String::from_utf8_lossy(&body).contains(reason) {
        panic!("rejection body lost: {body:?}");
    }
    let lines = capture.lines_with("request rejected");
    let [line] = lines.as_slice() else {
        panic!("expected one access line, got {}", capture.text());
    };
    if !line.contains("status=400") || !line.contains(reason) {
        panic!("access line misses the reason: {line}");
    }
}
