use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::config::RendererMode;
use immich_edit_backend::services::render::{RenderCacheOptions, RenderService};
use immich_edit_backend::state::AppState;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

const MB: u64 = 1024 * 1024;

fn preview(id: Uuid) -> Request<Body> {
    json_request(
        "POST",
        &format!("/api/assets/{id}/preview"),
        serde_json::json!({"max_edge": 512, "edits": {}}),
    )
}

fn warm(ids: &[Uuid]) -> Request<Body> {
    json_request(
        "POST",
        "/api/frames/warm",
        serde_json::json!({ "ids": ids }),
    )
}

fn with_frame_cache(state: &AppState, raw_frame_cache_mb: u64) -> AppState {
    let mut state = state.clone();
    state.render = RenderService::new(
        RenderCacheOptions {
            raw_frame_cache_mb,
            quality_frame_cache_mb: 256,
            gpu_texture_cache_mb: 256,
        },
        RendererMode::Cpu,
        false,
        state.rasters.clone(),
        state.luts.clone(),
        state.dcp.clone(),
    );
    state
}

#[tokio::test]
async fn a_warmed_neighbour_opens_from_the_frame_cache() {
    let server = MockServer::start().await;
    let current = asset_id();
    let next = Uuid::new_v4();
    mock_original(&server, current).await;
    mock_original(&server, next).await;
    let state = test_state(&server).await;
    let app = seed_and_wrap(&server, state.clone()).await;

    let resp = app.clone().oneshot(preview(current)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("current preview status {}", resp.status());
    }
    let resp = app.clone().oneshot(warm(&[next])).await.unwrap();
    if resp.status() != StatusCode::ACCEPTED {
        panic!("warm status {}", resp.status());
    }

    let one_frame = state.render.frame_cache_bytes().await.preview_used;
    let deadline = Instant::now() + Duration::from_secs(30);
    while state.render.frame_cache_bytes().await.preview_used <= one_frame {
        if Instant::now() > deadline {
            panic!("the neighbour frame was never warmed");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let resp = app.oneshot(preview(next)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("neighbour preview status {}", resp.status());
    }
    let fetched = originals_fetched(&server, next).await;
    let frames = state.render.telemetry().snapshot().frames;
    if fetched != 1 || frames.cache_misses != 2 || frames.cache_hits != 1 {
        panic!(
            "neighbour preview decoded again: {fetched} downloads, {} misses, {} hits",
            frames.cache_misses, frames.cache_hits
        );
    }
}

#[tokio::test]
async fn warming_never_crowds_out_the_open_frame() {
    let server = MockServer::start().await;
    let current = asset_id();
    let next = Uuid::new_v4();
    mock_original(&server, current).await;
    mock_original(&server, next).await;
    let base = test_state(&server).await;
    let probe = seed_and_wrap(&server, base.clone()).await;
    let resp = probe.oneshot(preview(current)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("probe preview status {}", resp.status());
    }
    let frame = base.render.frame_cache_bytes().await.preview_used;
    let two_and_a_half_frames_mb = frame * 5 / 2 / MB;

    for (raw_frame_cache_mb, open_first) in [(256, false), (two_and_a_half_frames_mb, true)] {
        let app = seed_and_wrap(&server, with_frame_cache(&base, raw_frame_cache_mb)).await;
        if open_first {
            let resp = app.clone().oneshot(preview(current)).await.unwrap();
            if resp.status() != StatusCode::OK {
                panic!("current preview status {}", resp.status());
            }
        }
        let resp = app.oneshot(warm(&[next])).await.unwrap();
        if resp.status() != StatusCode::NO_CONTENT {
            panic!(
                "{raw_frame_cache_mb} MB cache, open first {open_first}: warm answered {}",
                resp.status()
            );
        }
        let fetched = originals_fetched(&server, next).await;
        if fetched != 0 {
            panic!("{raw_frame_cache_mb} MB cache, open first {open_first}: warm downloaded");
        }
    }
}

#[tokio::test]
async fn a_newer_warm_replaces_one_still_waiting() {
    let server = MockServer::start().await;
    let current = asset_id();
    let slow = Uuid::new_v4();
    let skipped = Uuid::new_v4();
    let latest = Uuid::new_v4();
    mock_original(&server, current).await;
    mock_original_with(
        &server,
        slow,
        arw_response().set_delay(Duration::from_millis(500)),
    )
    .await;
    mock_original(&server, skipped).await;
    mock_original(&server, latest).await;
    let app = seed_and_wrap(&server, test_state(&server).await).await;

    let resp = app.clone().oneshot(preview(current)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("current preview status {}", resp.status());
    }
    for id in [slow, skipped, latest] {
        let resp = app.clone().oneshot(warm(&[id])).await.unwrap();
        if resp.status() != StatusCode::ACCEPTED {
            panic!("warm status {}", resp.status());
        }
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    while originals_fetched(&server, latest).await == 0 {
        if Instant::now() > deadline {
            panic!("the latest warm never ran");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let fetched = originals_fetched(&server, skipped).await;
    if fetched != 0 {
        panic!("a superseded warm still downloaded {fetched} times");
    }
}

#[tokio::test]
async fn warm_takes_one_or_two_assets() {
    let server = MockServer::start().await;
    let app = seed_and_wrap(&server, test_state(&server).await).await;
    for count in [0, 3] {
        let ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();
        let resp = app.clone().oneshot(warm(&ids)).await.unwrap();
        if resp.status() != StatusCode::BAD_REQUEST {
            panic!("{count} ids answered {}", resp.status());
        }
    }
}
