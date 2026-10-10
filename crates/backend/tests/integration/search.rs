use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn search_response(next_page: Option<&str>) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "assets": {
            "items": [{ "id": asset_id(), "type": "IMAGE" }],
            "count": 1, "total": 1, "nextPage": next_page
        }
    }))
}

#[tokio::test]
async fn smart_search_sends_the_text_to_immich() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/smart"))
        .and(body_partial_json(
            json!({ "query": "red car", "page": 1, "size": 100 }),
        ))
        .respond_with(search_response(None))
        .mount(&server)
        .await;

    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/search/smart",
            json!({ "text": "red car", "filter": {}, "size": 100 }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("smart search status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json["items"][0]["id"] != asset_id().to_string() || !json["nextCursor"].is_null() {
        panic!("smart search response {json}");
    }
}

#[tokio::test]
async fn metadata_pages_follow_the_returned_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(body_partial_json(
            json!({ "page": 1, "albumIds": [album_id()], "order": "asc" }),
        ))
        .respond_with(search_response(Some("2")))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(body_partial_json(json!({ "page": 2 })))
        .respond_with(search_response(None))
        .mount(&server)
        .await;

    let app = test_app(&server).await;
    let query = json!({
        "filter": { "scope": { "album": album_id() } },
        "sort": { "field": "taken", "direction": "asc" },
        "size": 1,
    });
    let first = body_json(
        app.clone()
            .oneshot(json_request("POST", "/api/search/metadata", query.clone()))
            .await
            .unwrap(),
    )
    .await;
    let Some(cursor) = first["nextCursor"].as_str() else {
        panic!("first page should carry a cursor: {first}");
    };
    let mut next = query;
    next["cursor"] = cursor.into();
    let second = body_json(
        app.oneshot(json_request("POST", "/api/search/metadata", next))
            .await
            .unwrap(),
    )
    .await;
    if !second["nextCursor"].is_null() || second["items"].as_array().map(Vec::len) != Some(1) {
        panic!("second page {second}");
    }
}

#[tokio::test]
async fn malformed_searches_are_rejected_before_immich() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for body in [
        json!({ "albumIds": [album_id()], "size": 10 }),
        json!({ "filter": {}, "size": 10, "cursor": "garbage!" }),
        json!({ "filter": {}, "size": 0 }),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/search/metadata", body.clone()))
            .await
            .unwrap();
        if !resp.status().is_client_error() {
            panic!("{body} gave {}", resp.status());
        }
    }
    if !server
        .received_requests()
        .await
        .unwrap_or_default()
        .is_empty()
    {
        panic!("a rejected search must not reach Immich");
    }
}
