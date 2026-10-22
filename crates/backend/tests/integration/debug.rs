use crate::common::*;
use axum::http::StatusCode;
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
