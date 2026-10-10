use crate::common::*;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn search_response(
    ids: &[Uuid],
    next_page: Option<&str>,
    next_cursor: Option<&str>,
) -> ResponseTemplate {
    let items: Vec<Value> = ids
        .iter()
        .map(|id| json!({ "id": id, "type": "IMAGE" }))
        .collect();
    ResponseTemplate::new(200).set_body_json(json!({
        "assets": {
            "items": items, "count": ids.len(), "total": ids.len(),
            "nextPage": next_page, "nextCursor": next_cursor
        }
    }))
}

async fn mount_search(
    server: &MockServer,
    route: &str,
    body: Value,
    response: ResponseTemplate,
    priority: u8,
) {
    Mock::given(method("POST"))
        .and(path(route))
        .and(body_partial_json(body))
        .respond_with(response)
        .with_priority(priority)
        .mount(server)
        .await;
}

async fn mount_library(server: &MockServer, tags: Value) {
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tags))
        .mount(server)
        .await;
}

async fn post(app: &axum::Router, uri: &str, body: Value) -> Value {
    let resp = app
        .clone()
        .oneshot(json_request("POST", uri, body.clone()))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("{uri} {body} gave {}", resp.status());
    }
    body_json(resp).await
}

fn ids(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn metadata_pages_follow_the_returned_cursor() {
    let flat_first = json!({ "page": 1, "albumIds": [album_id()], "order": "asc" });
    let structured_first = json!({
        "filter": { "albumIds": { "all": [album_id()] }, "trashedAt": { "eq": null } },
        "orderBy": { "field": "fileCreatedAt", "direction": "asc" },
    });
    for (minor, first, second, first_page) in [
        (
            1,
            flat_first,
            json!({ "page": 2 }),
            search_response(&[asset_id()], Some("2"), None),
        ),
        (
            2,
            structured_first,
            json!({ "cursor": "immich-next" }),
            search_response(&[asset_id()], None, Some("immich-next")),
        ),
    ] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        mount_search(&server, "/api/search/metadata", first, first_page, 5).await;
        let last_page = search_response(&[asset_id()], None, None);
        mount_search(&server, "/api/search/metadata", second, last_page, 2).await;

        let app = test_app(&server).await;
        let query = json!({
            "filter": { "scope": { "album": album_id() } },
            "sort": { "field": "taken", "direction": "asc" },
            "size": 1,
        });
        let first = post(&app, "/api/search/metadata", query.clone()).await;
        let Some(cursor) = first["nextCursor"].as_str() else {
            panic!("3.{minor} first page should carry a cursor: {first}");
        };
        let mut next = query;
        next["cursor"] = cursor.into();
        let second = post(&app, "/api/search/metadata", next).await;
        if !second["nextCursor"].is_null() || ids(&second) != [asset_id().to_string()] {
            panic!("3.{minor} second page {second}");
        }
    }
}

#[tokio::test]
async fn smart_search_pages_flat_on_every_server() {
    let [a, b, c] = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    for minor in [1, 2] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        let first = json!({ "query": "red car", "page": 1, "size": 2 });
        let first_page = search_response(&[a, b], Some("2"), None);
        mount_search(&server, "/api/search/smart", first, first_page, 5).await;
        let last_page = search_response(&[c], None, None);
        mount_search(
            &server,
            "/api/search/smart",
            json!({ "page": 2 }),
            last_page,
            2,
        )
        .await;

        let app = test_app(&server).await;
        let query = json!({ "text": "red car", "filter": {}, "size": 2 });
        let first = post(&app, "/api/search/smart", query.clone()).await;
        let Some(cursor) = first["nextCursor"].as_str() else {
            panic!("3.{minor} first page should carry a cursor: {first}");
        };
        if ids(&first) != [a.to_string(), b.to_string()] {
            panic!("3.{minor} first page {first}");
        }
        let mut next = query;
        next["cursor"] = cursor.into();
        let second = post(&app, "/api/search/smart", next).await;
        if !second["nextCursor"].is_null() || ids(&second) != [c.to_string()] {
            panic!("3.{minor} second page {second}");
        }
    }
}

#[tokio::test]
async fn statistics_count_with_the_server_shape() {
    for (minor, body) in [
        (1, json!({ "tagIds": [tag_id()] })),
        (2, json!({ "filter": { "tagIds": { "all": [tag_id()] } } })),
    ] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        let total = ResponseTemplate::new(200).set_body_json(json!({ "total": 7 }));
        mount_search(&server, "/api/search/statistics", body, total, 5).await;

        let app = test_app(&server).await;
        let query = json!({ "filter": { "scope": { "tag": tag_id() } } });
        let stats = post(&app, "/api/search/statistics", query).await;
        if stats["total"] != 7 {
            panic!("3.{minor} statistics {stats}");
        }
    }
}

#[tokio::test]
async fn a_minimum_rating_needs_structured_search() {
    for (minor, expected) in [(1, StatusCode::BAD_REQUEST), (2, StatusCode::OK)] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        let total = ResponseTemplate::new(200).set_body_json(json!({ "total": 4 }));
        let body = json!({ "filter": { "rating": { "gte": 3 } } });
        mount_search(&server, "/api/search/statistics", body, total, 5).await;

        let query = json!({ "filter": { "rating": { "gte": 3 } } });
        let resp = test_app(&server)
            .await
            .oneshot(json_request("POST", "/api/search/statistics", query))
            .await
            .unwrap();
        if resp.status() != expected {
            panic!("3.{minor} minimum rating gave {}", resp.status());
        }
    }
}

#[tokio::test]
async fn rejected_filters_use_the_reject_tag() {
    let reject = Uuid::new_v4();
    for (minor, rejected, body) in [
        (1, "only", Some(json!({ "tagIds": [reject] }))),
        (1, "hide", None),
        (
            2,
            "only",
            Some(json!({ "filter": { "tagIds": { "all": [reject] } } })),
        ),
        (
            2,
            "hide",
            Some(json!({ "filter": { "tagIds": { "none": [reject] } } })),
        ),
    ] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        mount_library(
            &server,
            json!([{ "id": reject, "name": "reject", "value": "immich-edit/reject" }]),
        )
        .await;
        if let Some(body) = &body {
            let total = ResponseTemplate::new(200).set_body_json(json!({ "total": 3 }));
            mount_search(&server, "/api/search/statistics", body.clone(), total, 5).await;
        }

        let query = json!({ "filter": { "rejected": rejected } });
        let resp = test_app(&server)
            .await
            .oneshot(json_request("POST", "/api/search/statistics", query))
            .await
            .unwrap();
        let expected = if body.is_some() {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        if resp.status() != expected {
            panic!("3.{minor} rejected {rejected} gave {}", resp.status());
        }
    }
}

#[tokio::test]
async fn only_rejected_without_a_reject_tag_is_empty() {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, 2, 2).await;
    mount_library(&server, json!([])).await;
    let app = test_app(&server).await;
    let filter = json!({ "rejected": "only" });

    let page = post(
        &app,
        "/api/search/metadata",
        json!({ "filter": filter, "size": 10 }),
    )
    .await;
    let stats = post(&app, "/api/search/statistics", json!({ "filter": filter })).await;

    if !ids(&page).is_empty() || !page["nextCursor"].is_null() || stats["total"] != 0 {
        panic!("only rejected without a reject tag gave {page} and {stats}");
    }
    let reached = server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .any(|request| request.url.path().starts_with("/api/search/"));
    if reached {
        panic!("a search that cannot match must not reach Immich");
    }
}

#[tokio::test]
async fn malformed_searches_are_rejected_before_immich() {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, 2, 2).await;
    let app = test_app(&server).await;
    let flat_cursor = URL_SAFE_NO_PAD.encode(br#"{"kind":"page","page":2}"#);
    for body in [
        json!({ "albumIds": [album_id()], "size": 10 }),
        json!({ "filter": {}, "size": 10, "cursor": "garbage!" }),
        json!({ "filter": {}, "size": 0 }),
        json!({ "filter": {}, "size": 10, "cursor": flat_cursor }),
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
    let reached = server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .any(|request| request.url.path().starts_with("/api/search/"));
    if reached {
        panic!("a rejected search must not reach Immich");
    }
}
