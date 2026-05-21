#![allow(dead_code)]

use immich_edit_backend::app;
use immich_edit_backend::config::{Config, RendererMode};
use immich_edit_backend::immich::ImmichClient;
use immich_edit_backend::services::edits_store::EditsStore;
use immich_edit_backend::services::preview_meta::PreviewMetaStore;
use immich_edit_backend::services::render::RenderService;
use immich_edit_backend::services::render_queue::RenderQueue;
use immich_edit_backend::state::AppState;
use std::sync::Arc;
use url::Url;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn test_state(server: &MockServer) -> AppState {
    let base = Url::parse(&server.uri()).unwrap();
    let immich = ImmichClient::new(base.clone(), "test-key").unwrap();
    let cache_dir = tempfile::tempdir().unwrap().keep();
    let config = Config {
        immich_url: base,
        immich_api_key: "test-key".into(),
        bind_addr: "127.0.0.1:0".into(),
        cache_dir: cache_dir.clone(),
        preview_max_edge: 1024,
        raw_frame_cache_mb: 64,
        linear_cache_mb: 64,
        render_max_concurrency: 1,
        renderer: RendererMode::Cpu,
    };
    AppState {
        config: Arc::new(config),
        immich: immich.clone(),
        edits: EditsStore::new(&cache_dir),
        render: RenderService::new(immich, 4),
        queue: RenderQueue::new(1),
        preview_meta: PreviewMetaStore::new(),
    }
}

pub fn router(state: AppState) -> axum::Router {
    app::router(state)
}

pub fn album_id() -> Uuid {
    Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap()
}

pub fn asset_id() -> Uuid {
    Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap()
}

pub async fn mock_albums(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": album_id(),
                "albumName": "Test Album",
                "assetCount": 3,
                "updatedAt": "2026-01-01T00:00:00Z"
            }
        ])))
        .mount(server)
        .await;
}

pub async fn mock_album_detail(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": album_id(),
            "albumName": "Test Album",
            "assetCount": 1,
            "assets": [{
                "id": asset_id(),
                "originalFileName": "DSC0001.ARW",
                "type": "IMAGE"
            }]
        })))
        .mount(server)
        .await;
}

pub async fn mock_thumb(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}/thumbnail", asset_id())))
        .and(query_param("size", "preview"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(vec![0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10]),
        )
        .mount(server)
        .await;
}

pub async fn mock_ping_ok(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/server/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_string("pong"))
        .mount(server)
        .await;
}
