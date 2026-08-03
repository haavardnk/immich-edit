#![cfg(feature = "segment")]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::MockServer;

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn models_list_requires_auth() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/masks/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn models_list_reports_catalog_and_install_state() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/masks/models")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["enabled"], false);
    assert_eq!(json["runtime"], "off");
    let models = json["models"].as_array().unwrap();
    assert!(!models.is_empty());

    let ormbg = models.iter().find(|m| m["id"] == "ormbg").unwrap();
    assert_eq!(ormbg["kind"], "subject");
    assert_eq!(ormbg["tier"], "recommended");
    assert_eq!(ormbg["license"], "Apache-2.0");
    assert_eq!(ormbg["installed"], false);
    assert_eq!(ormbg["installing"], false);
    assert_eq!(ormbg["progress_bytes"], 0);
    assert!(ormbg["install_error"].is_null());
    assert!(ormbg["size_bytes"].as_u64().unwrap() > 0);
    assert!(ormbg["gpu_mb"].as_u64().unwrap() > 0);

    assert_eq!(json["active"].as_object().unwrap().len(), 0);
}

#[tokio::test]
async fn selecting_unknown_model_is_not_found() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/admin/masks/default")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"subject","model_id":"nope"}"#.to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn selecting_model_of_wrong_kind_is_rejected() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/admin/masks/default")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"sky","model_id":"ormbg"}"#.to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn selecting_uninstalled_model_is_rejected() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/admin/masks/default")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"subject","model_id":"ormbg"}"#.to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn installing_unknown_model_is_not_found() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/models/not-a-real-model")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn removing_uninstalled_model_is_not_found() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/admin/models/ormbg")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
