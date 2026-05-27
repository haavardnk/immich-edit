#![allow(dead_code)]

use immich_edit_backend::app;
use immich_edit_backend::config::{Config, RendererMode};
use immich_edit_backend::immich::ImmichClient;
use immich_edit_backend::services::edited_thumb::EditedThumbService;
use immich_edit_backend::services::edits_store::EditsStore;
use immich_edit_backend::services::preview_meta::PreviewMetaStore;
use immich_edit_backend::services::raster_store::RasterStore;
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
        bind_socket: "127.0.0.1:0".parse().unwrap(),
        cache_dir: cache_dir.clone(),
        preview_max_edge: 1024,
        render_max_concurrency: 1,
        mask_cache_mb: 1024,
        renderer: RendererMode::Cpu,
        database_url: "sqlite::memory:".into(),
        auth_token: None,
        allowed_origins: Vec::new(),
        debug_endpoints: true,
        max_body_mb: 128,
        original_timeout_secs: 120,
        export_timeout_secs: 300,
        insecure: true,
    };
    let rasters = RasterStore::new(&cache_dir, 1024).unwrap();
    AppState {
        config: Arc::new(config),
        immich: immich.clone(),
        edits: EditsStore::migrated_memory().await.unwrap(),
        render: RenderService::new(immich, 4, RendererMode::Cpu, rasters.clone()),
        queue: RenderQueue::new(1),
        preview_meta: PreviewMetaStore::new(),
        edited_thumb: EditedThumbService::new(&cache_dir, 1).unwrap(),
        rasters,
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

pub async fn mock_asset_detail(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": asset_id(),
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "originalMimeType": "image/x-sony-arw",
            "fileCreatedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-02T00:00:00Z",
            "checksum": "abc",
            "isFavorite": true,
            "exifInfo": {
                "make": "SONY",
                "model": "ILCE-7M4",
                "lensModel": "FE 35mm F1.8",
                "fNumber": 2.8,
                "focalLength": 35.0,
                "iso": 400,
                "exposureTime": "0.004",
                "exifImageWidth": 4032,
                "exifImageHeight": 3024,
                "dateTimeOriginal": "2026-01-01T00:00:00Z",
                "rating": 4,
                "fileSizeInByte": 12345678u64
            },
            "tags": [
                { "id": "11111111-aaaa-bbbb-cccc-000000000001", "name": "Landscape", "value": "Landscape" }
            ]
        })))
        .mount(server)
        .await;
}

pub async fn mock_asset_update(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": asset_id(),
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "isFavorite": true,
            "exifInfo": { "rating": 5 },
            "tags": []
        })))
        .mount(server)
        .await;
}

pub async fn mock_tag_upsert(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path("/api/tags"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": "22222222-aaaa-bbbb-cccc-000000000002", "name": "New", "value": "New" }
        ])))
        .mount(server)
        .await;
}

pub fn tag_id() -> Uuid {
    Uuid::parse_str("33333333-aaaa-bbbb-cccc-000000000003").unwrap()
}

pub async fn mock_tag_asset(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path(format!("/api/tags/{}/assets", tag_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": asset_id(), "success": true }
        ])))
        .mount(server)
        .await;
}

pub async fn mock_untag_asset(server: &MockServer) {
    Mock::given(method("DELETE"))
        .and(path(format!("/api/tags/{}/assets", tag_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": asset_id(), "success": true }
        ])))
        .mount(server)
        .await;
}
