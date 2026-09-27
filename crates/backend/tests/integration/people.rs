use crate::common::*;
use axum::http::StatusCode;
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn people_list_includes_asset_counts() {
    let server = MockServer::start().await;
    mock_people_list_with_stats(&server, 17).await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/people")).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["assetCount"] != 17 {
        panic!("body: {json}");
    }
}
