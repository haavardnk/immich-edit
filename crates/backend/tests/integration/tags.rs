use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[tokio::test]
async fn tag_asset_reports_bulk_refusals() {
    for (verb, error, expected) in [
        ("PUT", "duplicate", StatusCode::OK),
        ("PUT", "no_permission", StatusCode::UNPROCESSABLE_ENTITY),
        ("DELETE", "not_found", StatusCode::OK),
        ("DELETE", "no_permission", StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let server = MockServer::start().await;
        Mock::given(method(verb))
            .and(path(format!("/api/tags/{}/assets", tag_id())))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                { "id": asset_id(), "success": false, "error": error }
            ])))
            .mount(&server)
            .await;
        let uri = format!("/api/tags/{}/assets/{}", tag_id(), asset_id());
        let resp = test_app(&server)
            .await
            .oneshot(json_request(verb, &uri, json!({})))
            .await
            .unwrap();
        if resp.status() != expected {
            panic!("{verb} {error}: status {}", resp.status());
        }
        if expected == StatusCode::OK {
            continue;
        }
        let json = body_json(resp).await;
        if json["code"] != "upstream_rejected"
            || json["message"] != "Immich refused the request: no permission"
        {
            panic!("{verb} {error}: body {json}");
        }
    }
}
