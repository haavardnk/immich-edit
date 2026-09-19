mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::Value;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = http_body_util::BodyExt::collect(res.into_body())
        .await
        .unwrap()
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn folder_paths_come_from_the_upstream_view() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/view/folder/unique-paths"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!(["2024", "2024/summer"])),
        )
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/folders/paths")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        body_json(res).await,
        serde_json::json!(["2024", "2024/summer"])
    );
}

#[tokio::test]
async fn folder_assets_forward_the_requested_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/view/folder"))
        .and(query_param("path", "2024/summer"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                "id": asset_id(),
                "originalFileName": "DSC0001.ARW",
                "type": "IMAGE"
            }])),
        )
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/folders/assets?path=2024%2Fsummer")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["id"], asset_id().to_string());
}

#[tokio::test]
async fn folder_assets_without_a_path_ask_for_the_root() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/view/folder"))
        .and(query_param("path", ""))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/folders/assets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await, serde_json::json!([]));
}
