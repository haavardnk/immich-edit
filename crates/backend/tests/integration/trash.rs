use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json as body_is, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_delete(server: &MockServer, ids: &[Uuid], response: ResponseTemplate, times: u64) {
    Mock::given(method("DELETE"))
        .and(path("/api/assets"))
        .and(body_is(json!({ "ids": ids, "force": false })))
        .respond_with(response)
        .expect(times)
        .mount(server)
        .await;
}

async fn post(server: &MockServer, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = test_app(server)
        .await
        .oneshot(json_request("POST", uri, body))
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

#[tokio::test]
async fn trash_soft_deletes_each_master_once() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, true).await;
    let id = asset_id();
    mock_delete(&server, &[id], ResponseTemplate::new(204), 1).await;

    let (status, body) = post(&server, "/api/trash", json!({ "ids": [id, id] })).await;
    if status != StatusCode::OK || body["count"] != 1 {
        panic!("trash: {status} {body}");
    }
}

#[tokio::test]
async fn trash_is_refused_while_immich_trash_is_off() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, false).await;
    let id = asset_id();
    mock_delete(&server, &[id], ResponseTemplate::new(204), 0).await;

    let (status, body) = post(&server, "/api/trash", json!({ "ids": [id] })).await;
    if status != StatusCode::CONFLICT || body["code"] != "trash_disabled" {
        panic!("trash off: {status} {body}");
    }
}

#[tokio::test]
async fn trash_is_refused_when_the_trash_setting_is_unreadable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/server/features"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let id = asset_id();
    mock_delete(&server, &[id], ResponseTemplate::new(204), 0).await;

    let (status, body) = post(&server, "/api/trash", json!({ "ids": [id] })).await;
    if status != StatusCode::BAD_GATEWAY || body["code"] != "upstream_unavailable" {
        panic!("features unreadable: {status} {body}");
    }
}

#[tokio::test]
async fn virtual_copies_are_not_sent_to_the_immich_trash() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, true).await;
    let id = asset_id();
    mock_delete(&server, &[id], ResponseTemplate::new(204), 0).await;

    let (status, body) = post(&server, "/api/trash", json!({ "ids": [format!("{id}_1")] })).await;
    if status != StatusCode::BAD_REQUEST {
        panic!("copy key: {status} {body}");
    }
}

#[tokio::test]
async fn a_key_without_delete_permission_is_a_clear_refusal() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, true).await;
    let id = asset_id();
    mock_delete(
        &server,
        &[id],
        ResponseTemplate::new(403).set_body_json(json!({
            "message": "Missing required permission: asset.delete",
            "error": "Forbidden",
            "statusCode": 403,
        })),
        1,
    )
    .await;

    let (status, body) = post(&server, "/api/trash", json!({ "ids": [id] })).await;
    let message = body["message"].as_str().unwrap_or_default();
    if status != StatusCode::UNPROCESSABLE_ENTITY
        || body["code"] != "upstream_rejected"
        || !message.contains("asset.delete")
    {
        panic!("missing permission: {status} {body}");
    }
}

#[tokio::test]
async fn restore_brings_masters_back_from_the_trash() {
    let server = MockServer::start().await;
    let id = asset_id();
    Mock::given(method("POST"))
        .and(path("/api/trash/restore/assets"))
        .and(body_is(json!({ "ids": [id] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "count": 1 })))
        .expect(1)
        .mount(&server)
        .await;

    let (status, body) = post(&server, "/api/trash/restore", json!({ "ids": [id] })).await;
    if status != StatusCode::OK || body["count"] != 1 {
        panic!("restore: {status} {body}");
    }
}
