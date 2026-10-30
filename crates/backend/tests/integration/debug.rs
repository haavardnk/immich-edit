use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn debug_timings_requires_admin() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app.oneshot(get("/api/debug/timings")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn debug_timings_served_to_admin() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/debug/timings")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["cache_bytes"]["denoise_disk_used"], 0);
    assert_eq!(body["cache_bytes"]["denoise_disk_cap"], 512 * 1024 * 1024);
}

#[tokio::test]
async fn debug_timings_report_the_render_plan() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    let app = test_app(&server).await;
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            json!({"max_edge": 512, "edits": {}}),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("preview status {}", resp.status());
    }
    let resp = app.oneshot(get("/api/debug/timings")).await.unwrap();
    let body = body_json(resp).await;
    let plans = body["plans"].as_array().cloned().unwrap_or_default();
    let [plan] = plans.as_slice() else {
        panic!("one render should record one plan: {plans:?}");
    };
    if plan["plan"] != "single_pass" || plan["renders"] != 1 || plan["last_tiles"] != 1 {
        panic!("plan stats wrong: {plan}");
    }
    if plan["renderer"] != body["renderer_active"] || !plan["edge"].is_null() {
        panic!(
            "plan renderer or edge wrong: {plan} with {}",
            body["renderer_active"]
        );
    }
}
