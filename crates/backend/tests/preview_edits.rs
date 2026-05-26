mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

fn req_get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn get_edits_returns_default_when_missing() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let id = asset_id();
    let resp = app
        .oneshot(req_get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if !json["manifest"]["ops"]
        .as_object()
        .map(|m| m.is_empty())
        .unwrap_or(false)
    {
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
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "originalFileName": "x.arw",
            "type": "IMAGE",
            "updatedAt": "2026-05-01T00:00:00Z",
            "checksum": "deadbeef"
        })))
        .mount(&server)
        .await;
    let state = test_state(&server).await;
    let app = router(state);

    let put_body = serde_json::json!({
        "schema_version": 2,
        "ops": {
            "exposure": { "ev": 1.5 },
            "transform": { "rotate": 90 }
        }
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .body(Body::from(put_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put status {}", resp.status());
    }
    let saved: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if saved["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("saved: {saved}");
    }
    if saved["immich_checksum"] != "deadbeef" {
        panic!("checksum metadata: {saved}");
    }

    let resp = app
        .clone()
        .oneshot(req_get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    let got: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if got["manifest"]["ops"]["transform"]["rotate"] != 90 {
        panic!("get: {got}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/assets/{id}/edits"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }

    let resp = app
        .oneshot(req_get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    let after: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if !after["manifest"]["ops"]
        .as_object()
        .map(|m| m.is_empty())
        .unwrap_or(false)
    {
        panic!("post-delete identity: {after}");
    }
}

fn arw_fixture() -> Option<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/sample.arw");
    std::fs::read(&path).ok()
}

async fn mock_arw_original(server: &MockServer, id: uuid::Uuid) {
    let bytes = arw_fixture().expect("sample.arw fixture required");
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(bytes),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn live_preview_renders_jpeg_and_returns_meta_id() {
    if arw_fixture().is_none() {
        eprintln!("sample.arw missing, skipping");
        return;
    }
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = router(test_state(&server).await);

    let body = serde_json::json!({"max_edge": 512, "edits": {"basic": {"exposure_ev": 1.0}}});
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/preview"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !ct.starts_with("image/jpeg") {
        panic!("content-type: {ct}");
    }
    let meta_id = resp
        .headers()
        .get("x-preview-meta-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let meta_id = match meta_id {
        Some(s) => s,
        None => panic!("missing meta header"),
    };
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg");
    }

    let resp = app
        .oneshot(req_get(&format!("/api/assets/{id}/preview/meta/{meta_id}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("meta status {}", resp.status());
    }
    let meta: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if meta["width"].as_u64().unwrap_or(0) == 0 {
        panic!("meta dims: {meta}");
    }
    let bins = meta["histogram"]["l"].as_array().unwrap();
    if bins.len() != 256 {
        panic!("histogram bins: {}", bins.len());
    }
}

#[tokio::test]
async fn live_preview_rejects_bad_max_edge() {
    let server = MockServer::start().await;
    let id = asset_id();
    let app = router(test_state(&server).await);
    let body = serde_json::json!({"max_edge": 10});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/preview"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn export_returns_full_res_jpeg() {
    if arw_fixture().is_none() {
        eprintln!("sample.arw missing, skipping");
        return;
    }
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = router(test_state(&server).await);

    let body = serde_json::json!({"edits": {}});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/export"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let disp = resp
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !disp.contains("attachment") {
        panic!("disposition: {disp}");
    }
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg");
    }
    if bytes.len() < 100_000 {
        panic!("full res suspiciously small: {} bytes", bytes.len());
    }
}
