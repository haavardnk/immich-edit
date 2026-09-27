use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn health_returns_ok_with_redacted_config() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let app = test_app(&server).await;

    let resp = app.oneshot(get("/api/health")).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    let s = json.to_string();
    if s.contains(TEST_API_KEY) {
        panic!("api key leaked: {s}");
    }
    if json["renderer_mode"] != "cpu" {
        panic!("renderer_mode field");
    }
    if json["renderer_active"] != "cpu" {
        panic!("renderer_active field");
    }
    if json["immich_reachable"] != true {
        panic!("ping flag");
    }
    if json["immich_status"]["kind"] != "ok" {
        panic!("immich status: {}", json["immich_status"]);
    }
    if json["host"]["cores"].as_u64().unwrap_or(0) == 0 {
        panic!("host cores: {}", json["host"]);
    }
    if json["host"]["arch"].as_str().unwrap_or("").is_empty() {
        panic!("host arch: {}", json["host"]);
    }
}

#[tokio::test]
async fn health_reports_specific_immich_failure() {
    for (status, kind, code) in [
        (401u16, "api_key_rejected", Value::Null),
        (503u16, "upstream_5xx", json!(503)),
    ] {
        let server = MockServer::start().await;
        mock_ping_status(&server, status).await;
        let app = test_app(&server).await;

        let resp = app.oneshot(get("/api/health")).await.unwrap();
        if resp.status() != StatusCode::OK {
            panic!("status {} for upstream {status}", resp.status());
        }
        let json = body_json(resp).await;
        if json["immich_reachable"] != false {
            panic!("ping flag for upstream {status}: {json}");
        }
        if json["immich_status"]["kind"] != kind {
            panic!("kind for upstream {status}: {json}");
        }
        if json["immich_status"]["status_code"] != code {
            panic!("status_code for upstream {status}: {json}");
        }
    }
}

#[tokio::test]
async fn live_endpoint_works_without_auth() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app.oneshot(get("/api/health/live")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["status"], "ok");
}
