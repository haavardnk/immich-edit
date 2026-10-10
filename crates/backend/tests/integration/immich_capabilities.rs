use crate::common::*;
use axum::http::StatusCode;
use serde_json::Value;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn capabilities(server: &MockServer) -> Value {
    let resp = test_app(server)
        .await
        .oneshot(get("/api/immich/capabilities"))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("capabilities returned {}", resp.status());
    }
    body_json(resp).await
}

#[tokio::test]
async fn structured_search_needs_immich_3_2() {
    for (major, minor, supported) in [(3, 0, false), (3, 1, false), (3, 2, true), (4, 0, true)] {
        let server = MockServer::start().await;
        mock_server_version(&server, major, minor, 1).await;
        let body = capabilities(&server).await;
        if body["structured_search"] != supported
            || body["immich_version"] != format!("{major}.{minor}.1")
        {
            panic!("{major}.{minor}: {body}");
        }
    }
}

#[tokio::test]
async fn an_unreadable_version_falls_back_to_flat_search() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/server/version"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let body = capabilities(&server).await;
    if body["structured_search"] != false || !body["immich_version"].is_null() {
        panic!("unknown version: {body}");
    }
}
