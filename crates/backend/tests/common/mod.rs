#![allow(dead_code)]

use immich_edit_backend::app;
use immich_edit_backend::config::{Config, MlRuntimeMode, RendererMode};
use immich_edit_backend::immich::client::ImmichUser;
use immich_edit_backend::services::asset_counts::AssetCountCache;
use immich_edit_backend::services::auth_store::{AuthKind, AuthStore};
use immich_edit_backend::services::crypto::InstanceCrypto;
use immich_edit_backend::services::dcp_store::DcpStore;
use immich_edit_backend::services::edited_thumb::EditedThumbService;
use immich_edit_backend::services::edits_store::EditsStore;
#[cfg(feature = "ml")]
use immich_edit_backend::services::embedding_cache::EmbeddingCache;
use immich_edit_backend::services::instance_store::InstanceStore;
use immich_edit_backend::services::job_store::JobStore;
use immich_edit_backend::services::login_limiter::LoginLimiter;
use immich_edit_backend::services::lut_store::LutStore;
#[cfg(feature = "ml")]
use immich_edit_backend::services::model_install::ModelInstaller;
#[cfg(feature = "ml")]
use immich_edit_backend::services::model_store::ModelStore;
use immich_edit_backend::services::preview_meta::PreviewMetaStore;
use immich_edit_backend::services::raster_store::RasterStore;
use immich_edit_backend::services::render::{RenderCacheOptions, RenderService};
use immich_edit_backend::services::render_queue::RenderQueue;
#[cfg(feature = "ml")]
use immich_edit_backend::services::segment::SegmentService;
use immich_edit_backend::state::AppState;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const TEST_API_KEY: &str = "test-key";

pub fn test_user_id() -> Uuid {
    Uuid::parse_str("99999999-8888-7777-6666-555555555555").unwrap()
}

pub async fn test_state(server: &MockServer) -> AppState {
    let cache_dir = tempfile::tempdir().unwrap().keep();
    let config = Config {
        bind_addr: "127.0.0.1:0".into(),
        bind_socket: "127.0.0.1:0".parse().unwrap(),
        data_dir: cache_dir.clone(),
        cache_dir: cache_dir.clone(),
        preview_max_edge: 1024,
        render_max_concurrency: 1,
        thumb_max_concurrency: 1,
        mask_cache_mb: 1024,
        embedding_cache_mb: 512,
        raw_frame_cache_mb: 256,
        quality_frame_cache_mb: 256,
        gpu_texture_cache_mb: 256,
        renderer: RendererMode::Cpu,
        database_url: "sqlite::memory:".into(),
        allowed_origins: Vec::new(),
        max_body_mb: 128,
        original_timeout_secs: 120,
        export_timeout_secs: 300,
        ml_runtime: MlRuntimeMode::Off,
        ml_max_edge: 2048,
        ml_max_concurrency: 1,
        ml_idle_secs: 60,
    };
    let _ = server;
    let edits = EditsStore::migrated_memory().await.unwrap();
    let rasters = RasterStore::new(&cache_dir, 1024, edits.pool())
        .await
        .unwrap();
    let instance = InstanceStore::new(edits.pool());
    let crypto =
        Arc::new(InstanceCrypto::load_or_create(&cache_dir.join("instance.key"), false).unwrap());
    let auth = AuthStore::new(edits.pool(), crypto.clone());
    let login_limiter = Arc::new(LoginLimiter::new());
    let luts = LutStore::new(edits.pool(), &cache_dir).unwrap();
    let dcp = DcpStore::new(edits.pool(), &cache_dir).unwrap();
    let jobs = JobStore::new(edits.pool(), crypto.clone());
    #[cfg(feature = "ml")]
    let models = ModelStore::new(edits.pool(), &cache_dir).unwrap();
    #[cfg(feature = "ml")]
    let installs = ModelInstaller::new(models.clone());
    #[cfg(feature = "ml")]
    let embeddings = EmbeddingCache::new(&cache_dir, 512).unwrap();
    #[cfg(feature = "ml")]
    let segment = SegmentService::new(&config, models.clone(), embeddings);
    AppState {
        config: Arc::new(config),
        crypto,
        instance,
        auth,
        login_limiter,
        edits,
        jobs,
        render: RenderService::new(
            RenderCacheOptions {
                raw_frame_cache_mb: 256,
                quality_frame_cache_mb: 256,
                gpu_texture_cache_mb: 256,
            },
            RendererMode::Cpu,
            rasters.clone(),
            luts.clone(),
            dcp.clone(),
        ),
        queue: RenderQueue::new(1),
        preview_meta: PreviewMetaStore::new(),
        edited_thumb: EditedThumbService::new(&cache_dir, 1).unwrap(),
        rasters,
        luts,
        dcp,
        #[cfg(feature = "ml")]
        models,
        #[cfg(feature = "ml")]
        installs,
        #[cfg(feature = "ml")]
        segment,
        tag_counts: AssetCountCache::new("tagIds"),
        people_counts: AssetCountCache::new("personIds"),
    }
}

pub fn router(state: AppState) -> axum::Router {
    app::router(state)
}

pub async fn seed_session(server: &MockServer, state: &AppState) -> String {
    state.instance.claim(&server.uri()).await.unwrap();
    let cfg = state.instance.get().await.unwrap();
    let user = ImmichUser {
        id: test_user_id(),
        email: "admin@test.local".into(),
        name: "Admin".into(),
        is_admin: true,
    };
    let rec = state.auth.upsert_user(&user).await.unwrap();
    state
        .auth
        .create_session(
            rec.id,
            AuthKind::ApiKey,
            TEST_API_KEY.as_bytes(),
            cfg.server_epoch,
            None,
            None,
        )
        .await
        .unwrap()
}

pub async fn seed_and_wrap(server: &MockServer, state: AppState) -> axum::Router {
    let token = seed_session(server, &state).await;
    wrap_auth(app::router(state), token)
}

pub async fn test_app(server: &MockServer) -> axum::Router {
    seed_and_wrap(server, test_state(server).await).await
}

fn wrap_auth(app: axum::Router, token: String) -> axum::Router {
    use axum::extract::Request;
    use axum::http::HeaderValue;
    use axum::middleware::{self, Next};
    app.layer(middleware::from_fn(move |mut req: Request, next: Next| {
        let cookie = format!("immich_edit_auth={token}");
        async move {
            if let Ok(v) = HeaderValue::from_str(&cookie) {
                req.headers_mut().insert("cookie", v);
            }
            if !req.headers().contains_key("host") {
                req.headers_mut()
                    .insert("host", HeaderValue::from_static("localhost"));
            }
            if !req.headers().contains_key("origin") {
                req.headers_mut()
                    .insert("origin", HeaderValue::from_static("http://localhost"));
            }
            next.run(req).await
        }
    }))
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

pub async fn mock_ping_status(server: &MockServer, status: u16) {
    Mock::given(method("GET"))
        .and(path("/api/server/ping"))
        .respond_with(ResponseTemplate::new(status))
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

pub async fn mock_tag_list_with_stats(server: &MockServer, count: u64) {
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": tag_id(), "name": "Blue", "value": "Blue" }
        ])))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": count
        })))
        .mount(server)
        .await;
}

pub fn person_id() -> Uuid {
    Uuid::parse_str("44444444-aaaa-bbbb-cccc-000000000004").unwrap()
}

pub async fn mock_people_list_with_stats(server: &MockServer, count: u64) {
    Mock::given(method("GET"))
        .and(path("/api/people"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "people": [ { "id": person_id(), "name": "Alice" } ],
            "total": 1,
            "hasNextPage": false
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": count
        })))
        .mount(server)
        .await;
}
