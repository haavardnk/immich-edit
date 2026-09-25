mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn arw_fixture() -> Vec<u8> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    std::fs::read(&file).expect("committed Sony ARW fixture")
}

async fn mock_original_owned_by_admin(server: &MockServer, id: uuid::Uuid) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(arw_fixture()),
        )
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", MEMBER_API_KEY))
        .respond_with(ResponseTemplate::new(403))
        .mount(server)
        .await;
}

fn post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn cached_preview_frame_is_not_served_to_another_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original_owned_by_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let body = serde_json::json!({"max_edge": 512, "edits": {}});
    let resp = admin
        .oneshot(post_json(
            &format!("/api/assets/{id}/preview"),
            body.clone(),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("owner preview status {}", resp.status());
    }

    let resp = member
        .oneshot(post_json(&format!("/api/assets/{id}/preview"), body))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_GATEWAY {
        panic!("member preview status {}", resp.status());
    }
}

#[tokio::test]
async fn cached_quality_frame_is_not_served_to_another_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original_owned_by_admin(&server, id).await;
    mock_asset_detail(&server).await;
    let (admin, member) = two_owner_apps(&server).await;

    let body = serde_json::json!({"edits": {}});
    let resp = admin
        .oneshot(post_json(&format!("/api/assets/{id}/export"), body.clone()))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("owner export status {}", resp.status());
    }

    let resp = member
        .oneshot(post_json(&format!("/api/assets/{id}/export"), body))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_GATEWAY {
        panic!("member export status {}", resp.status());
    }
}
