use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::config::Config;
use immich_edit_backend::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SLOW: Duration = Duration::from_secs(2);

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

#[tokio::test]
async fn export_outlives_the_light_request_timeout() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original_with(&server, id, arw_response().set_delay(SLOW)).await;
    mock_asset_detail(&server).await;
    let app = seed_and_wrap(&server, slow_state(&server).await).await;

    let resp = app
        .oneshot(get(&format!(
            "/api/assets/{id}/export?format=jpeg&quality=90"
        )))
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
    let state = slow_state(&server).await;
    let telemetry = state.render.telemetry().clone();
    let app = seed_and_wrap(&server, state).await;

    let resp = app.oneshot(get("/api/albums")).await.unwrap();

    if resp.status() != StatusCode::REQUEST_TIMEOUT {
        panic!("expected 408, got {}", resp.status());
    }
    let timeouts = telemetry.snapshot().request_timeouts;
    if timeouts != 1 {
        panic!("diagnostics counted {timeouts} timed-out requests, expected 1");
    }
}
