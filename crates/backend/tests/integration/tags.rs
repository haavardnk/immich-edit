use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::{body_json as body_is, header, method, path};
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

async fn send(
    server: &MockServer,
    verb: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = match body {
        Some(body) => json_request(verb, uri, body),
        None => empty_request(verb, uri),
    };
    let resp = test_app(server).await.oneshot(request).await.unwrap();
    let status = resp.status();
    let bytes = body_bytes(resp).await;
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn never(server: &MockServer, verb: &str) {
    Mock::given(method(verb))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(server)
        .await;
}

#[tokio::test]
async fn creating_a_tag_sends_the_trimmed_name_parent_and_color() {
    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    Mock::given(method("POST"))
        .and(path("/api/tags"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_is(json!({
            "name": "Svalbard",
            "parentId": child_tag_id(),
            "color": "#46a758"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "33333333-aaaa-bbbb-cccc-000000000009",
            "name": "Svalbard",
            "value": "Travel/Norway/Svalbard",
            "parentId": child_tag_id(),
            "color": "#46a758"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let (status, tag) = send(
        &server,
        "POST",
        "/api/tags",
        Some(json!({ "name": " Svalbard ", "parentId": child_tag_id(), "color": "46A758" })),
    )
    .await;
    if status != StatusCode::OK
        || tag["value"] != "Travel/Norway/Svalbard"
        || tag["assetCount"] != 0
    {
        panic!("{status}: {tag}");
    }
}

#[tokio::test]
async fn creating_a_tag_rejects_invalid_names_and_colors_before_immich() {
    for body in [
        json!({ "name": "  " }),
        json!({ "name": "Travel/Norway" }),
        json!({ "name": "x".repeat(81) }),
        json!({ "name": "Norway", "color": "blue" }),
        json!({ "name": "Norway", "order": 1 }),
        json!({ "name": "Svalbard", "parentId": "33333333-aaaa-bbbb-cccc-00000000000f" }),
    ] {
        let server = MockServer::start().await;
        mock_tag_tree(&server).await;
        never(&server, "POST").await;
        let (status, _) = send(&server, "POST", "/api/tags", Some(body.clone())).await;
        if !status.is_client_error() {
            panic!("{body} returned {status}");
        }
    }
}

#[tokio::test]
async fn managed_tags_cannot_be_created_changed_or_deleted() {
    let reject = format!("/api/tags/{}", managed_tag_id());
    for (verb, uri, body) in [
        (
            "POST",
            "/api/tags".to_string(),
            Some(json!({ "name": "Immich-Edit" })),
        ),
        (
            "POST",
            "/api/tags".to_string(),
            Some(json!({ "name": "label", "parentId": managed_tag_id() })),
        ),
        ("PATCH", reject.clone(), Some(json!({ "color": "#e5484d" }))),
        ("DELETE", reject.clone(), None),
    ] {
        let server = MockServer::start().await;
        mock_tag_tree(&server).await;
        mock_server_version(&server, 3, 3, 0).await;
        for write in ["POST", "PUT", "DELETE"] {
            never(&server, write).await;
        }
        let (status, json) = send(&server, verb, &uri, body).await;
        if status != StatusCode::BAD_REQUEST
            || json["message"] != "immich-edit tags are managed by immich-edit"
        {
            panic!("{verb} {uri}: {status} {json}");
        }
    }
}

#[tokio::test]
async fn a_sibling_with_the_same_name_is_a_conflict() {
    for (verb, uri, body) in [
        (
            "POST",
            "/api/tags".to_string(),
            json!({ "name": "norway", "parentId": tag_id() }),
        ),
        ("POST", "/api/tags".to_string(), json!({ "name": "TRAVEL" })),
    ] {
        let server = MockServer::start().await;
        mock_tag_tree(&server).await;
        never(&server, "POST").await;
        let (status, json) = send(&server, verb, &uri, Some(body)).await;
        if status != StatusCode::CONFLICT
            || !json["message"]
                .as_str()
                .unwrap_or("")
                .ends_with("already exists")
        {
            panic!("{verb} {uri}: {status} {json}");
        }
    }
}

#[tokio::test]
async fn an_immich_duplicate_on_create_is_a_conflict() {
    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    Mock::given(method("POST"))
        .and(path("/api/tags"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "message": "A tag with that name already exists" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let (status, json) = send(
        &server,
        "POST",
        "/api/tags",
        Some(json!({ "name": "Film" })),
    )
    .await;
    if status != StatusCode::CONFLICT || json["message"] != "A tag with that name already exists" {
        panic!("{status}: {json}");
    }
}

async fn mock_tag_update(server: &MockServer, forwarded: Value) {
    Mock::given(method("PUT"))
        .and(path(format!("/api/tags/{}", child_tag_id())))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_is(forwarded))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": child_tag_id(),
            "name": "Norge",
            "value": "Travel/Norge",
            "parentId": tag_id()
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn renaming_a_tag_needs_immich_3_3() {
    let uri = format!("/api/tags/{}", child_tag_id());
    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    mock_server_version(&server, 3, 3, 0).await;
    mock_tag_update(&server, json!({ "name": "Norge" })).await;
    let (status, tag) = send(&server, "PATCH", &uri, Some(json!({ "name": " Norge " }))).await;
    if status != StatusCode::OK || tag["value"] != "Travel/Norge" {
        panic!("3.3 rename: {status} {tag}");
    }

    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    mock_server_version(&server, 3, 2, 2).await;
    never(&server, "PUT").await;
    let (status, json) = send(&server, "PATCH", &uri, Some(json!({ "name": "Norge" }))).await;
    if status != StatusCode::BAD_REQUEST
        || json["message"] != "renaming tags needs Immich 3.3 or later"
    {
        panic!("3.2 rename: {status} {json}");
    }
}

#[tokio::test]
async fn a_color_change_works_on_any_version_and_null_clears_it() {
    let uri = format!("/api/tags/{}", child_tag_id());
    for (sent, forwarded) in [
        (json!({ "color": "#E5484D" }), json!({ "color": "#e5484d" })),
        (json!({ "color": null }), json!({ "color": null })),
    ] {
        let server = MockServer::start().await;
        mock_tag_tree(&server).await;
        mock_server_version(&server, 3, 0, 0).await;
        mock_tag_update(&server, forwarded).await;
        let (status, json) = send(&server, "PATCH", &uri, Some(sent.clone())).await;
        if status != StatusCode::OK {
            panic!("{sent}: {status} {json}");
        }
    }
}

#[tokio::test]
async fn renaming_checks_siblings_but_not_the_tag_itself() {
    let uri = format!("/api/tags/{}", child_tag_id());
    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    mock_server_version(&server, 3, 3, 0).await;
    mock_tag_update(&server, json!({ "name": "NORWAY" })).await;
    let (status, json) = send(&server, "PATCH", &uri, Some(json!({ "name": "NORWAY" }))).await;
    if status != StatusCode::OK {
        panic!("case-only rename: {status} {json}");
    }

    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    mock_server_version(&server, 3, 3, 0).await;
    never(&server, "PUT").await;
    let travel = format!("/api/tags/{}", tag_id());
    let (status, json) = send(
        &server,
        "PATCH",
        &travel,
        Some(json!({ "name": "immich-edit" })),
    )
    .await;
    if status != StatusCode::BAD_REQUEST {
        panic!("top-level managed rename: {status} {json}");
    }
}

#[tokio::test]
async fn updating_a_tag_rejects_an_empty_unknown_or_missing_tag() {
    for (uri, body, expected) in [
        (
            format!("/api/tags/{}", child_tag_id()),
            json!({}),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/tags/{}", child_tag_id()),
            json!({ "parentId": tag_id() }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "/api/tags/33333333-aaaa-bbbb-cccc-00000000000f".to_string(),
            json!({ "color": "#fff" }),
            StatusCode::NOT_FOUND,
        ),
    ] {
        let server = MockServer::start().await;
        mock_tag_tree(&server).await;
        never(&server, "PUT").await;
        let (status, json) = send(&server, "PATCH", &uri, Some(body.clone())).await;
        if status != expected {
            panic!("{uri} {body}: {status} {json}");
        }
    }
}

#[tokio::test]
async fn deleting_a_tag_recounts_every_tag() {
    let server = MockServer::start().await;
    mock_tag_tree(&server).await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "total": 3 })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("/api/tags/{}", child_tag_id())))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let statistics = || async {
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .filter(|r| r.url.path() == "/api/search/statistics")
            .count()
    };
    for _ in 0..2 {
        let resp = app.clone().oneshot(get("/api/tags")).await.unwrap();
        if resp.status() != StatusCode::OK {
            panic!("list {}", resp.status());
        }
    }
    let counted = statistics().await;
    let resp = app
        .clone()
        .oneshot(empty_request(
            "DELETE",
            &format!("/api/tags/{}", child_tag_id()),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete {}", resp.status());
    }
    app.oneshot(get("/api/tags")).await.unwrap();
    let recounted = statistics().await;
    if counted == 0 || recounted != counted * 2 {
        panic!("statistics before {counted}, after {recounted}");
    }
}
