use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use immich_edit_backend::services::export::BatchExecutor;
use immich_edit_backend::services::job_runner::JobExecutor;
use immich_edit_backend::services::job_store::JobItemRecord;
use immich_edit_backend::state::AppState;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::{body_json, json_request, mock_original_with, plain_jpeg, test_user_id};

pub fn preset_variant(preset: &str, params: Value) -> Value {
    json!({ "preset": preset, "params": params })
}

pub fn create_job_request(kind: &str, params: Value, asset_ids: &[String]) -> Request<Body> {
    json_request(
        "POST",
        "/api/jobs",
        json!({ "kind": kind, "params": params, "asset_ids": asset_ids }),
    )
}

pub async fn created_job_id(resp: Response) -> Uuid {
    let status = resp.status();
    let body = body_json(resp).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["id"].as_str().unwrap().parse().unwrap()
}

pub fn named_asset_detail(id: Uuid, name: &str) -> Value {
    json!({
        "id": id,
        "originalFileName": name,
        "type": "IMAGE",
        "ownerId": test_user_id()
    })
}

pub async fn mock_jpeg_original(server: &MockServer, id: Uuid) {
    mock_original_with(
        server,
        id,
        ResponseTemplate::new(200)
            .insert_header("content-type", "image/jpeg")
            .set_body_bytes(plain_jpeg()),
    )
    .await;
}

pub async fn mock_named_asset(server: &MockServer, id: Uuid, name: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(named_asset_detail(id, name)))
        .mount(server)
        .await;
    mock_jpeg_original(server, id).await;
}

pub async fn run_item(state: &AppState, item: JobItemRecord) -> Value {
    let job = state.jobs.get_job(item.job_id).await.unwrap().unwrap();
    let result = BatchExecutor::new(state.clone())
        .execute(job, item.clone())
        .await
        .unwrap_or_else(|e| panic!("item {} variant {}: {e}", item.position, item.variant));
    state.jobs.complete_item(item.id, &result).await.unwrap();
    result
}

pub async fn run_all(state: &AppState) -> Vec<Value> {
    let mut results = Vec::new();
    while let Some(item) = state.jobs.claim_next_item().await.unwrap() {
        results.push(run_item(state, item).await);
    }
    results
}
