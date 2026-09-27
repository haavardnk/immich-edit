mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::json;
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn tags_upsert_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_tag_upsert(&server).await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request("PUT", "/api/tags", json!({"tags": ["New"]})))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["value"] != "New" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn tags_list_includes_asset_counts() {
    let server = MockServer::start().await;
    mock_tag_list_with_stats(&server, 42).await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/tags")).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["assetCount"] != 42 {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn tag_asset_add_and_remove_proxy() {
    let server = MockServer::start().await;
    mock_tag_asset(&server).await;
    mock_untag_asset(&server).await;
    let app = test_app(&server).await;
    let uri = format!("/api/tags/{}/assets/{}", tag_id(), asset_id());

    let resp = app
        .clone()
        .oneshot(json_request("PUT", &uri, json!({})))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["success"] != true {
        panic!("body: {json}");
    }

    let resp = app
        .oneshot(json_request("DELETE", &uri, json!({})))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("delete status {}", resp.status());
    }
}
