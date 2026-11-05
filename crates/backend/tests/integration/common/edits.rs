use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use super::http::{body_json, json_request};

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
