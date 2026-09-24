mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use immich_edit_backend::config::Config;
use immich_edit_backend::services::render_queue::RenderQueue;
use immich_edit_backend::state::AppState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn arw_fixture() -> Vec<u8> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    std::fs::read(&file).expect("committed Sony ARW fixture")
}

async fn mock_original(server: &MockServer, id: uuid::Uuid, delay: Duration) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(arw_fixture())
                .set_delay(delay),
        )
        .mount(server)
        .await;
}

async fn impatient_state(server: &MockServer) -> AppState {
    let mut state = test_state(server).await;
    state.config = Arc::new(Config {
        request_timeout_secs: 1,
        ..(*state.config).clone()
    });
    state
}

async fn parallel_state(server: &MockServer) -> AppState {
    let mut state = test_state(server).await;
    state.queue = RenderQueue::new(4, 4);
    state
}

fn preview_lane(id: uuid::Uuid, lane: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/assets/{id}/preview"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"max_edge": 512, "edits": {}, "lane": lane}).to_string(),
        ))
        .unwrap()
}

fn preview(id: uuid::Uuid) -> Request<Body> {
    preview_lane(id, "base")
}

async fn originals_fetched(server: &MockServer, id: uuid::Uuid) -> usize {
    let wanted = format!("/api/assets/{id}/original");
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter(|r| r.url.path() == wanted)
        .count()
}

#[tokio::test]
async fn concurrent_previews_download_the_original_once() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id, Duration::from_millis(200)).await;
    let app = seed_and_wrap(&server, parallel_state(&server).await).await;

    let first = app.clone().oneshot(preview_lane(id, "base"));
    let second = app.oneshot(preview_lane(id, "roi"));
    let (first, second) = tokio::join!(first, second);

    let first = first.unwrap();
    let second = second.unwrap();
    if first.status() != StatusCode::OK || second.status() != StatusCode::OK {
        panic!("preview status {} and {}", first.status(), second.status());
    }
    let fetched = originals_fetched(&server, id).await;
    if fetched != 1 {
        panic!("expected a single original download, got {fetched}");
    }
}

#[tokio::test]
async fn a_timed_out_preview_keeps_the_decoded_frame() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id, Duration::from_millis(1500)).await;
    let state = impatient_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token.clone());

    let resp = app.oneshot(preview(id)).await.unwrap();
    if resp.status() != StatusCode::REQUEST_TIMEOUT {
        panic!(
            "expected the first preview to time out, got {}",
            resp.status()
        );
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    while state.render.frame_cache_bytes().await.preview_used == 0 {
        if Instant::now() > deadline {
            panic!("the timed-out request discarded the decoded frame");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let mut patient = state.clone();
    patient.config = Arc::new(Config {
        request_timeout_secs: 60,
        ..(*state.config).clone()
    });
    let resp = wrap_auth(router(patient), token)
        .oneshot(preview(id))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("retry after the timeout failed: {}", resp.status());
    }
    let fetched = originals_fetched(&server, id).await;
    if fetched != 1 {
        panic!("expected the retry to reuse the cached frame, got {fetched} downloads");
    }

    let snap = state.render.telemetry().snapshot();
    let frames = snap.frames;
    if frames.cache_misses != 1 || frames.cache_hits != 1 {
        panic!(
            "expected one frame cache miss then one hit, got {} misses and {} hits",
            frames.cache_misses, frames.cache_hits
        );
    }
    if frames.fetch.count != 1 || frames.decode.count != 1 || frames.fetch.p50_us < 1_000_000 {
        panic!("frame load timing not recorded once around the slow download: {frames:?}");
    }
    if snap.request_timeouts != 1 {
        panic!(
            "expected one timed-out request, got {}",
            snap.request_timeouts
        );
    }
    let stages: Vec<&str> = snap
        .stages
        .iter()
        .filter(|s| s.renderer == "cpu")
        .map(|s| s.stage)
        .collect();
    if !stages.contains(&"demosaic") || !stages.contains(&"encode") {
        panic!("cpu stage timings missing from diagnostics: {stages:?}");
    }
}
