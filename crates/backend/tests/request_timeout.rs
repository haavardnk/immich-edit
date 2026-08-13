mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use immich_edit_backend::config::Config;
use immich_edit_backend::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SLOW: Duration = Duration::from_secs(2);

fn arw_fixture() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    std::fs::read(&path).expect("committed Sony ARW fixture")
}

async fn slow_state(server: &MockServer) -> AppState {
    let mut state = test_state(server).await;
    state.config = Arc::new(Config {
        request_timeout_secs: 1,
        original_timeout_secs: 30,
        export_timeout_secs: 30,
        ..(*state.config).clone()
    });
    state
}

async fn mock_slow_original(server: &MockServer, id: uuid::Uuid, bytes: Vec<u8>) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(bytes)
                .set_delay(SLOW),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn export_outlives_the_light_request_timeout() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_slow_original(&server, id, arw_fixture()).await;
    let app = seed_and_wrap(&server, slow_state(&server).await).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/export?format=jpeg&quality=90"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if resp.status() != StatusCode::OK {
        panic!("export cut short: {}", resp.status());
    }
}

#[tokio::test]
async fn light_routes_still_time_out() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([]))
                .set_delay(SLOW),
        )
        .mount(&server)
        .await;
    let app = seed_and_wrap(&server, slow_state(&server).await).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/albums")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if resp.status() != StatusCode::REQUEST_TIMEOUT {
        panic!("expected 408, got {}", resp.status());
    }
}
