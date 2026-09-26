mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn capabilities(server: &MockServer) -> Value {
    let app = test_app(server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/immich/capabilities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("capabilities returned {}", resp.status());
    }
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn the_minimum_rating_filter_needs_immich_3_2() {
    for (major, minor, supported) in [(3, 0, false), (3, 1, false), (3, 2, true), (4, 0, true)] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/server/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "major": major,
                "minor": minor,
                "patch": 1,
                "prerelease": null
            })))
            .mount(&server)
            .await;
        let body = capabilities(&server).await;
        if body["min_rating_filter"] != supported
            || body["immich_version"] != format!("{major}.{minor}.1")
        {
            panic!("{major}.{minor}: {body}");
        }
    }
}

#[tokio::test]
async fn an_unknown_immich_version_disables_the_minimum_rating_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/server/version"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let body = capabilities(&server).await;
    if body["min_rating_filter"] != false || !body["immich_version"].is_null() {
        panic!("unknown version: {body}");
    }
}
