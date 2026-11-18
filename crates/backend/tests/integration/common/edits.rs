use std::fmt::Display;

use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use super::http::{body_json, get, json_request};

pub fn exposure_manifest(ev: f64) -> Value {
    json!({ "schema_version": 2, "ops": { "exposure": { "ev": ev } } })
}

pub async fn put_edits(app: &axum::Router, id: Uuid, body: Value) -> Value {
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

pub async fn history(app: &axum::Router, key: impl Display) -> Vec<Value> {
    let resp = app
        .clone()
        .oneshot(get(&format!("/api/assets/{key}/edits/history")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("history status {}", resp.status());
    }
    body_json(resp)
        .await
        .as_array()
        .cloned()
        .expect("history array")
}

pub async fn history_entry_id(app: &axum::Router, key: impl Display, hash: &Value) -> i64 {
    history(app, key)
        .await
        .iter()
        .find(|e| &e["manifest_hash"] == hash)
        .and_then(|e| e["id"].as_i64())
        .expect("history entry id")
}

pub async fn restore_history(app: &axum::Router, key: impl Display, entry_id: i64) -> Response {
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{key}/edits/restore"),
            json!({ "entry_id": entry_id }),
        ))
        .await
        .unwrap()
}
