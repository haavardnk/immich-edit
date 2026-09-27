mod common;

use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use common::*;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

fn exposure_manifest(ev: f64) -> Value {
    json!({ "schema_version": 2, "ops": { "exposure": { "ev": ev } } })
}

async fn put_edits(app: &axum::Router, id: Uuid, body: Value) -> Value {
    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            body,
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put: {}", resp.status());
    }
    body_json(resp).await
}

async fn history_entry_id(app: &axum::Router, id: Uuid, hash: &str) -> i64 {
    let resp = app
        .clone()
        .oneshot(get(&format!("/api/assets/{id}/edits/history")))
        .await
        .unwrap();
    let history = body_json(resp).await;
    history
        .as_array()
        .and_then(|arr| arr.iter().find(|e| e["manifest_hash"] == hash))
        .and_then(|e| e["id"].as_i64())
        .expect("history entry id")
}

fn with_if_match(mut req: Request<Body>, value: &str) -> Request<Body> {
    req.headers_mut()
        .insert("if-match", HeaderValue::from_str(value).unwrap());
    req
}

fn is_identity(doc: &Value) -> bool {
    doc["manifest"]["ops"]
        .as_object()
        .is_some_and(|m| m.is_empty())
}

#[tokio::test]
async fn get_edits_returns_default_when_missing() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = asset_id();
    let resp = app
        .oneshot(get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if !is_identity(&json) {
        panic!("default document not empty: {json}");
    }
    if json["asset_id"].as_str() != Some(&id.to_string()) {
        panic!("asset id: {json}");
    }
}

#[tokio::test]
async fn put_then_get_then_delete_edits() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let state = test_state(&server).await;
    let app = seed_and_wrap(&server, state).await;
    let uri = format!("/api/assets/{id}/edits");

    let saved = put_edits(
        &app,
        id,
        json!({
            "schema_version": 2,
            "ops": {
                "exposure": { "ev": 1.5 },
                "transform": { "rotate": 90 }
            }
        }),
    )
    .await;
    if saved["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("saved: {saved}");
    }
    if saved["immich_checksum"] != "abc" {
        panic!("checksum metadata: {saved}");
    }

    let got = body_json(app.clone().oneshot(get(&uri)).await.unwrap()).await;
    if got["manifest"]["ops"]["transform"]["rotate"] != 90 {
        panic!("get: {got}");
    }

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &uri))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }

    let after = body_json(app.oneshot(get(&uri)).await.unwrap()).await;
    if !is_identity(&after) {
        panic!("post-delete identity: {after}");
    }
}

#[tokio::test]
async fn put_with_if_match_conflict_returns_current() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let uri = format!("/api/assets/{id}/edits");

    let saved = put_edits(&app, id, exposure_manifest(1.5)).await;
    let current_hash = saved["hash"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(with_if_match(
            json_request("PUT", &uri, exposure_manifest(2.0)),
            "stale-hash",
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::CONFLICT {
        panic!("expected 409, got {}", resp.status());
    }
    let body = body_json(resp).await;
    if body["hash"].as_str() != Some(current_hash.as_str()) {
        panic!("conflict body hash: {body}");
    }

    let resp = app
        .oneshot(with_if_match(
            json_request("PUT", &uri, exposure_manifest(2.0)),
            &current_hash,
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("matching if-match should succeed: {}", resp.status());
    }
}

#[tokio::test]
async fn delete_with_if_match_conflict_returns_current() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let uri = format!("/api/assets/{id}/edits");

    let saved = put_edits(&app, id, exposure_manifest(1.5)).await;
    let current_hash = saved["hash"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(with_if_match(empty_request("DELETE", &uri), "stale-hash"))
        .await
        .unwrap();
    if resp.status() != StatusCode::CONFLICT {
        panic!("expected 409, got {}", resp.status());
    }
    let body = body_json(resp).await;
    if body["hash"].as_str() != Some(current_hash.as_str()) {
        panic!("conflict body hash: {body}");
    }

    let kept = body_json(app.clone().oneshot(get(&uri)).await.unwrap()).await;
    if kept["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("conflicting delete must not reset: {kept}");
    }

    let resp = app
        .oneshot(with_if_match(
            empty_request("DELETE", &uri),
            &format!("\"{current_hash}\""),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("matching if-match should delete: {}", resp.status());
    }
}

#[tokio::test]
async fn list_edits_stays_lean_unless_assets_are_requested() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    put_edits(&app, id, exposure_manifest(1.5)).await;

    let lean = body_json(app.clone().oneshot(get("/api/edits")).await.unwrap()).await;
    if !lean["render_revision"]
        .as_str()
        .is_some_and(|r| !r.is_empty())
    {
        panic!("list lost its render revision: {lean}");
    }
    let lean = &lean["items"];
    let entry = lean[0].as_object().expect("entry object");
    let mut keys: Vec<&str> = entry.keys().map(String::as_str).collect();
    keys.sort_unstable();
    if keys != ["hash", "id", "updated_at"] {
        panic!("default list shape changed: {lean}");
    }

    let rich = body_json(
        app.oneshot(get("/api/edits?with_assets=true"))
            .await
            .unwrap(),
    )
    .await;
    let rich = &rich["items"];
    if rich[0]["asset"]["originalFileName"] != "DSC0001.ARW" {
        panic!("enriched list: {rich}");
    }
    if rich[0]["hash"] != lean[0]["hash"] {
        panic!("enriched entry lost its hash: {rich}");
    }
}

#[tokio::test]
async fn put_writes_history_revision() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, exposure_manifest(1.5)).await;

    let resp = app
        .oneshot(get(&format!("/api/assets/{id}/edits/history")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("history status {}", resp.status());
    }
    let entries = body_json(resp).await;
    let arr = entries.as_array().unwrap();
    if arr.len() != 2 {
        panic!("expected 2 history entries, got {}", arr.len());
    }
    if arr[0]["edits"]["basic"]["exposure_ev"] != 1.5 {
        panic!("newest first: {entries}");
    }
    if arr[1]["edits"]["basic"]["exposure_ev"] != 0.5 {
        panic!("oldest second: {entries}");
    }
}

#[tokio::test]
async fn restore_returns_previous_edits() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, exposure_manifest(1.5)).await;
    let entry_id = history_entry_id(&app, id, first["hash"].as_str().unwrap()).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/edits/restore"),
            json!({ "entry_id": entry_id }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored = body_json(resp).await;
    if restored["manifest"]["ops"]["exposure"]["ev"] != 0.5 {
        panic!("restore body: {restored}");
    }

    let current = body_json(
        app.oneshot(get(&format!("/api/assets/{id}/edits")))
            .await
            .unwrap(),
    )
    .await;
    if current["manifest"]["ops"]["exposure"]["ev"] != 0.5 {
        panic!("get-after-restore: {current}");
    }
}

#[tokio::test]
async fn restore_after_reset_refreshes_upstream_meta() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let saved = put_edits(&app, id, exposure_manifest(0.5)).await;

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }

    let entry_id = history_entry_id(&app, id, saved["hash"].as_str().unwrap()).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/edits/restore"),
            json!({ "entry_id": entry_id }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored = body_json(resp).await;
    if restored["immich_checksum"] != "abc" || restored["immich_updated_at"].is_null() {
        panic!("restore meta: {restored}");
    }
}

#[tokio::test]
async fn auto_edits_rejects_malformed_body() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{}/edits/auto", Uuid::new_v4()))
                .header("content-type", "application/json")
                .body(Body::from("{ not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn white_balance_sample_rejects_out_of_range_point() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{}/edits/white-balance", Uuid::new_v4()),
            json!({ "u": 1.5, "v": 0.5 }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn white_balance_auto_rejects_malformed_body() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/assets/{}/edits/white-balance/auto",
                    Uuid::new_v4()
                ))
                .header("content-type", "application/json")
                .body(Body::from("{ not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}
