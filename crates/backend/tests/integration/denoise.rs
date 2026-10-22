use std::time::Duration;

use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::services::purge;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::Level;
use uuid::Uuid;
use wiremock::{MockServer, ResponseTemplate};

async fn denoise(app: &axum::Router, method: &str) -> Value {
    denoise_call(app, asset_id(), method).await
}

async fn settle(app: &axum::Router) -> Value {
    settle_denoise(app, asset_id()).await
}

#[tokio::test]
async fn denoise_requires_auth() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/denoise", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn denoise_reports_disabled_and_missing_models() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    assert_eq!(denoise(&app, "GET").await, json!({"state": "disabled"}));
    assert_eq!(denoise(&app, "POST").await, json!({"state": "disabled"}));

    let state = ml_enabled(test_state(&server).await);
    let app = seed_and_wrap(&server, state).await;
    assert_eq!(
        denoise(&app, "GET").await,
        json!({"state": "model_missing"})
    );
    assert_eq!(
        denoise(&app, "POST").await,
        json!({"state": "model_missing"})
    );
}

#[tokio::test]
async fn a_bake_that_cannot_run_settles_with_the_reason() {
    let cases = [
        (
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(plain_jpeg()),
            json!({"state": "unsupported"}),
        ),
        (
            ResponseTemplate::new(503),
            json!({"state": "failed", "error": "The photo could not be downloaded from Immich."}),
        ),
    ];
    for (original, settled) in cases {
        let server = MockServer::start().await;
        mock_original_with(&server, asset_id(), original).await;
        let state = ml_enabled(test_state(&server).await);
        install_fake_model(&state).await;
        let app = seed_and_wrap(&server, state).await;
        assert_eq!(denoise(&app, "GET").await, json!({"state": "idle"}));
        let started = denoise(&app, "POST").await;
        if !matches!(started["state"].as_str(), Some("queued" | "running")) && started != settled {
            panic!("start: {started}");
        }
        assert_eq!(settle(&app).await, settled);
    }
}

#[tokio::test]
async fn a_broken_model_fails_with_a_short_message_and_can_be_retried() {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    let state = ml_enabled(test_state(&server).await);
    install_fake_model(&state).await;
    let app = seed_and_wrap(&server, state).await;
    let (logs, _guard) = capture_logs(Level::WARN);
    denoise(&app, "POST").await;
    let failed = settle(&app).await;
    assert_eq!(
        failed,
        json!({"state": "failed", "error": "Denoising failed. Check the server log."})
    );
    let logged = logs.lines_with("AI denoise bake failed");
    assert!(
        logged
            .iter()
            .any(|line| line.contains("loading the denoise model")),
        "the cause chain must reach the log: {logged:?}"
    );
    let retried = denoise(&app, "POST").await;
    if !matches!(
        retried["state"].as_str(),
        Some("queued" | "running" | "failed")
    ) {
        panic!("retry: {retried}");
    }
    assert_eq!(settle(&app).await["state"], "failed");
}

#[tokio::test]
async fn a_bake_waits_for_the_shared_inference_permit_between_tiles() {
    let server = MockServer::start().await;
    let warm = Uuid::from_u128(7);
    mock_original(&server, warm).await;
    mock_original(&server, asset_id()).await;
    let (state, permits) = with_denoise(test_state(&server).await, |_| {});
    install_identity_model(&state).await;
    let app = seed_and_wrap(&server, state).await;
    denoise_call(&app, warm, "POST").await;
    assert_eq!(settle_denoise(&app, warm).await, json!({"state": "ready"}));

    let held = permits.acquire().await.unwrap();
    denoise(&app, "POST").await;
    let mut tiling = Value::Null;
    for _ in 0..1200 {
        tiling = denoise(&app, "GET").await;
        if tiling["total"].as_u64().is_some_and(|total| total > 0) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(
        tiling["state"] == "running" && tiling["done"] == 0,
        "tiling must start and stop at the first tile: {tiling}"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    let blocked = denoise(&app, "GET").await;
    assert_eq!(
        blocked, tiling,
        "no tile may run while another inference holds the only permit"
    );
    drop(held);
    assert_eq!(settle(&app).await, json!({"state": "ready"}));
}

#[tokio::test]
async fn an_owner_cannot_queue_unbounded_bakes() {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    let (state, permits) = with_denoise(test_state(&server).await, |_| {});
    install_fake_model(&state).await;
    let app = seed_and_wrap(&server, state).await;

    let held = permits.acquire().await.unwrap();
    let mut sources = vec![asset_id()];
    sources.extend((1..8u128).map(Uuid::from_u128));
    for source in &sources {
        let started = denoise_call(&app, *source, "POST").await;
        assert!(
            matches!(started["state"].as_str(), Some("queued" | "running")),
            "{source}: {started}"
        );
    }
    let resp = app
        .clone()
        .oneshot(empty_request(
            "POST",
            &format!("/api/assets/{}/denoise", Uuid::from_u128(99)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(header_str(&resp, "retry-after").as_deref(), Some("30"));
    assert_eq!(body_json(resp).await["code"], "rate_limited");
    let again = denoise_call(&app, asset_id(), "POST").await;
    assert!(
        matches!(again["state"].as_str(), Some("queued" | "running")),
        "an already queued photo still answers: {again}"
    );
    drop(held);
    assert_eq!(settle(&app).await["state"], "failed");
}

#[tokio::test]
async fn a_bake_that_finishes_after_a_purge_keeps_nothing() {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    let (state, permits) = with_denoise(test_state(&server).await, |_| {});
    install_identity_model(&state).await;
    let token = seed_session(&server, &state).await;
    let identity = identity_for(&state, test_user_id()).await;
    let key = plane_key(identity, asset_id());
    let app = wrap_auth(router(state.clone()), token);
    let (logs, _guard) = capture_logs(Level::WARN);

    let held = permits.acquire().await.unwrap();
    denoise(&app, "POST").await;
    wait_for_denoise_state(&app, asset_id(), "running").await;
    purge::purge_owner(&state, identity.owner).await.unwrap();
    drop(held);

    for _ in 0..1200 {
        if !logs.lines_with("AI denoise bake failed").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let failed = logs.lines_with("AI denoise bake failed");
    assert!(
        failed.iter().any(|line| line.contains("purged")),
        "the bake must notice the purge: {failed:?}"
    );
    let store = state.render.denoise().store();
    assert!(
        !store.contains(&key).await,
        "a purged owner got a plane back"
    );
    assert_eq!(store.disk_bytes().await.0, 0);
    assert_eq!(
        serde_json::to_value(state.denoise.status(identity, asset_id()).await).unwrap(),
        json!({"state": "idle"})
    );
}
