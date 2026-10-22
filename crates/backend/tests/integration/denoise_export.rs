use std::time::Duration;

use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::Level;
use wiremock::MockServer;

fn export_request() -> axum::http::Request<axum::body::Body> {
    json_request(
        "POST",
        &format!("/api/assets/{}/export", asset_id()),
        json!({"edits": {"detail": {"ai_denoise": true}}, "metadata": "none"}),
    )
}

async fn export(app: &axum::Router) -> axum::response::Response {
    app.clone().oneshot(export_request()).await.unwrap()
}

fn warnings(resp: &axum::response::Response) -> Option<Value> {
    resp.headers()
        .get("x-export-warnings")
        .map(|v| serde_json::from_slice(v.as_bytes()).unwrap())
}

async fn mocked_server() -> MockServer {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    mock_asset_detail(&server).await;
    server
}

#[tokio::test]
async fn an_export_waits_for_the_bake_and_then_succeeds() {
    let server = mocked_server().await;
    let (state, permits) = with_denoise(test_state(&server).await, |_| {});
    install_identity_model(&state).await;
    let app = seed_and_wrap(&server, state).await;

    let held = permits.acquire().await.unwrap();
    let pending = tokio::spawn({
        let app = app.clone();
        async move { export(&app).await }
    });
    wait_for_denoise_state(&app, asset_id(), "running").await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!pending.is_finished(), "the export must wait for the bake");
    drop(held);

    let resp = pending.await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(warnings(&resp), None);
    assert_eq!(
        denoise_call(&app, asset_id(), "GET").await,
        json!({"state": "ready"})
    );
}

#[tokio::test]
async fn an_export_after_a_failed_bake_is_an_internal_error() {
    let server = mocked_server().await;
    let state = ml_enabled(test_state(&server).await);
    install_fake_model(&state).await;
    let app = seed_and_wrap(&server, state).await;
    let (logs, _guard) = capture_logs(Level::ERROR);

    let resp = export(&app).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_json(resp).await;
    assert_eq!(body["code"], "internal");
    assert_eq!(body["message"], "internal error");
    assert!(
        !logs.lines_with("loading the denoise model").is_empty(),
        "the failure must be logged with its context: {}",
        logs.text()
    );
}

#[tokio::test]
async fn an_export_gives_up_on_a_bake_that_outlives_the_export_timeout() {
    let server = mocked_server().await;
    let (state, permits) = with_denoise(test_state(&server).await, |config| {
        config.export_timeout_secs = 1;
    });
    install_fake_model(&state).await;
    let app = seed_and_wrap(&server, state).await;
    let (logs, _guard) = capture_logs(Level::WARN);

    let held = permits.acquire().await.unwrap();
    let resp = export(&app).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body = body_json(resp).await;
    assert_eq!(body["code"], "conflict");
    assert_eq!(
        body["message"],
        "AI denoise is still running. Try again when it finishes."
    );
    assert!(
        !logs
            .lines_with("export gave up waiting for the AI denoise bake")
            .is_empty(),
        "a timed-out wait must be logged: {}",
        logs.text()
    );
    drop(held);
}

#[tokio::test]
async fn an_export_warns_when_ai_denoise_is_skipped() {
    let server = mocked_server().await;
    let disabled = test_app(&server).await;
    let resp = export(&disabled).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        warnings(&resp),
        Some(json!([
            "AI denoise skipped: machine learning is disabled on this server."
        ]))
    );

    let missing = seed_and_wrap(&server, ml_enabled(test_state(&server).await)).await;
    let resp = export(&missing).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        warnings(&resp),
        Some(json!([
            "AI denoise skipped: no denoise model is installed."
        ]))
    );
}

#[tokio::test]
async fn an_export_uses_a_stored_plane_like_the_preview_does() {
    let server = mocked_server().await;
    let state = test_state(&server).await;
    install_identity_model(&state).await;
    let token = seed_session(&server, &state).await;
    let identity = identity_for(&state, test_user_id()).await;
    put_grey_plane(&state, plane_key(identity, asset_id())).await;
    let app = wrap_auth(router(state.clone()), token);

    let resp = export(&app).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(warnings(&resp), None, "a stored plane is used with ML off");
    let denoised = body_bytes(resp).await;

    state.render.denoise().store().purge_all().await;
    let resp = export(&app).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        warnings(&resp),
        Some(json!([
            "AI denoise skipped: machine learning is disabled on this server."
        ]))
    );
    assert_ne!(
        body_bytes(resp).await,
        denoised,
        "the plane must reach the render"
    );
}
