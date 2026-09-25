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
    let app = test_app(&server).await;
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
    let app = seed_and_wrap(&server, state).await;

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

#[tokio::test]
async fn put_with_if_match_conflict_returns_current() {
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
    let app = test_app(&server).await;

    let first = serde_json::json!({
        "schema_version": 2,
        "ops": { "exposure": { "ev": 1.5 } }
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .body(Body::from(first.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("first put: {}", resp.status());
    }
    let saved: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let current_hash = saved["hash"].as_str().unwrap().to_string();

    let second = serde_json::json!({
        "schema_version": 2,
        "ops": { "exposure": { "ev": 2.0 } }
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .header("if-match", "stale-hash")
                .body(Body::from(second.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::CONFLICT {
        panic!("expected 409, got {}", resp.status());
    }
    let body: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if body["hash"].as_str() != Some(current_hash.as_str()) {
        panic!("conflict body hash: {body}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .header("if-match", &current_hash)
                .body(Body::from(second.to_string()))
                .unwrap(),
        )
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
    let app = test_app(&server).await;

    let put_body = serde_json::json!({
        "schema_version": 2,
        "ops": { "exposure": { "ev": 1.5 } }
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
        panic!("put: {}", resp.status());
    }
    let saved: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let current_hash = saved["hash"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/assets/{id}/edits"))
                .header("if-match", "stale-hash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::CONFLICT {
        panic!("expected 409, got {}", resp.status());
    }
    let body: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if body["hash"].as_str() != Some(current_hash.as_str()) {
        panic!("conflict body hash: {body}");
    }

    let resp = app
        .clone()
        .oneshot(req_get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    let kept: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if kept["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("conflicting delete must not reset: {kept}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/assets/{id}/edits"))
                .header("if-match", format!("\"{current_hash}\""))
                .body(Body::empty())
                .unwrap(),
        )
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
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "originalFileName": "beach.arw",
            "type": "IMAGE",
            "updatedAt": "2026-05-01T00:00:00Z",
            "checksum": "deadbeef"
        })))
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let put_body = serde_json::json!({
        "schema_version": 2,
        "ops": { "exposure": { "ev": 1.5 } }
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
        panic!("put: {}", resp.status());
    }

    let resp = app.clone().oneshot(req_get("/api/edits")).await.unwrap();
    let lean: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let entry = lean[0].as_object().expect("entry object");
    let mut keys: Vec<&str> = entry.keys().map(String::as_str).collect();
    keys.sort_unstable();
    if keys != ["hash", "id", "updated_at"] {
        panic!("default list shape changed: {lean}");
    }

    let resp = app
        .oneshot(req_get("/api/edits?with_assets=true"))
        .await
        .unwrap();
    let rich: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if rich[0]["asset"]["originalFileName"] != "beach.arw" {
        panic!("enriched list: {rich}");
    }
    if rich[0]["hash"] != lean[0]["hash"] {
        panic!("enriched entry lost its hash: {rich}");
    }
}

fn arw_fixture() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    std::fs::read(&path).expect("committed Sony ARW fixture")
}

async fn mock_arw_original(server: &MockServer, id: uuid::Uuid) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(arw_fixture()),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn live_preview_renders_jpeg_and_returns_meta_id() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

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

fn luma_dc_quantizer(jpeg: &[u8]) -> u8 {
    let mut at = 2;
    while jpeg[at + 1] != 0xdb {
        at += 2 + usize::from(u16::from_be_bytes([jpeg[at + 2], jpeg[at + 3]]));
    }
    jpeg[at + 5]
}

#[tokio::test]
async fn live_previews_encode_lighter_than_persisted_ones() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let persisted = app
        .clone()
        .oneshot(req_get(&format!("/api/assets/{id}/preview?max=512")))
        .await
        .unwrap();
    let persisted = body_bytes(persisted).await;
    let live = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/preview"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({"max_edge": 512}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let live = body_bytes(live).await;

    let quantizers = (luma_dc_quantizer(&persisted), luma_dc_quantizer(&live));
    if quantizers != (2, 3) {
        panic!("expected quality 95 then 90 (luma DC 2 then 3), got {quantizers:?}");
    }
}

#[tokio::test]
async fn live_preview_rejects_bad_max_edge() {
    let server = MockServer::start().await;
    let id = asset_id();
    let app = test_app(&server).await;
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
async fn live_preview_with_clip_warn_skips_meta() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let body = serde_json::json!({"max_edge": 512, "clip_warn": true});
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
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    if resp.headers().contains_key("x-preview-meta-id") {
        panic!("clip warn render must not publish preview meta");
    }
}

#[tokio::test]
async fn persisted_preview_etag_varies_with_clip_flag() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let etag_for = async |clip: bool| {
        let resp = app
            .clone()
            .oneshot(req_get(&format!(
                "/api/assets/{id}/preview?max=512&clip={clip}"
            )))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("status {}", resp.status());
        }
        resp.headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
            .expect("etag")
    };
    let plain = etag_for(false).await;
    let clipped = etag_for(true).await;
    if plain == clipped {
        panic!("clip flag must change the etag: {plain}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/preview?max=512&clip=true"))
                .header("if-none-match", plain.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("stale etag must not revalidate: {}", resp.status());
    }
}

#[tokio::test]
async fn export_returns_full_res_jpeg() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let body = serde_json::json!({"edits": {}, "filename_template": "{date}_{name}"});
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
    if disp
        != "attachment; filename=\"2026-01-01_DSC0001.jpg\"; filename*=UTF-8''2026-01-01_DSC0001.jpg"
    {
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

async fn export_frame(body: serde_json::Value) -> raw_pipeline::frame::RawFrame {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
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
        panic!("status {} for {body}", resp.status());
    }
    raw_pipeline::decode::decode(&body_bytes(resp).await).unwrap()
}

async fn export_dims(body: serde_json::Value) -> (usize, usize) {
    let frame = export_frame(body).await;
    (frame.meta.width, frame.meta.height)
}

fn neighbour_contrast(frame: &raw_pipeline::frame::RawFrame) -> f64 {
    frame
        .data
        .iter()
        .zip(&frame.data[frame.cpp..])
        .map(|(a, b)| f64::from((b - a).abs()))
        .sum()
}

#[tokio::test]
async fn export_output_sharpening_raises_local_contrast() {
    let plain = export_frame(serde_json::json!({
        "edits": {},
        "format": "png",
        "resize_mode": "dimensions",
        "resize_width": 600,
        "resize_height": 600
    }))
    .await;
    let sharpened = export_frame(serde_json::json!({
        "edits": {},
        "format": "png",
        "resize_mode": "dimensions",
        "resize_width": 600,
        "resize_height": 600,
        "output_sharpen_media": "matte",
        "output_sharpen_amount": "high"
    }))
    .await;
    if (plain.meta.width, plain.meta.height) != (sharpened.meta.width, sharpened.meta.height) {
        panic!("output sharpening changed the size");
    }
    let before = neighbour_contrast(&plain);
    let after = neighbour_contrast(&sharpened);
    if after <= before * 1.05 {
        panic!("sharpened contrast {after} vs plain {before}");
    }
}

#[tokio::test]
async fn export_rejects_bad_output_sharpening_density() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{}/export", asset_id()))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "edits": {},
                        "output_sharpen_media": "glossy",
                        "output_sharpen_ppi": 20
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn export_fits_the_requested_dimensions() {
    let boxed = export_dims(serde_json::json!({
        "edits": {},
        "resize_mode": "dimensions",
        "resize_width": 800,
        "resize_height": 800
    }))
    .await;
    if boxed.0.max(boxed.1) != 800 {
        panic!("800x800 export is {boxed:?}");
    }
    let wide = export_dims(serde_json::json!({
        "edits": {},
        "resize_mode": "dimensions",
        "resize_width": 400
    }))
    .await;
    if wide.0 != 400 {
        panic!("400 wide export is {wide:?}");
    }
}

#[tokio::test]
async fn export_enlarges_a_small_crop_only_when_asked() {
    let edits = serde_json::json!({
        "geometry": { "crop": { "x": 0.4, "y": 0.4, "w": 0.1, "h": 0.1 } }
    });
    let kept = export_dims(serde_json::json!({
        "edits": edits,
        "resize_mode": "dimensions",
        "resize_width": 1500,
        "resize_height": 1500
    }))
    .await;
    if kept.0.max(kept.1) >= 1500 {
        panic!("crop was enlarged without asking: {kept:?}");
    }
    let enlarged = export_dims(serde_json::json!({
        "edits": edits,
        "resize_mode": "dimensions",
        "resize_width": 1500,
        "resize_height": 1500,
        "resize_enlarge": true
    }))
    .await;
    if enlarged.0.max(enlarged.1) != 1500 {
        panic!("enlarged crop is {enlarged:?}");
    }
}

#[tokio::test]
async fn export_rejects_a_resize_without_a_size() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{}/export", asset_id()))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "edits": {}, "resize_mode": "dimensions" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

async fn mock_asset_metadata(server: &MockServer, id: uuid::Uuid, checksum: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "originalFileName": "x.arw",
            "type": "IMAGE",
            "updatedAt": "2026-05-01T00:00:00Z",
            "checksum": checksum,
        })))
        .mount(server)
        .await;
}

async fn put_edits(
    app: &axum::Router,
    id: uuid::Uuid,
    body: serde_json::Value,
) -> serde_json::Value {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put: {}", resp.status());
    }
    serde_json::from_slice(&body_bytes(resp).await).unwrap()
}

#[tokio::test]
async fn put_writes_history_revision() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_metadata(&server, id, "abc").await;
    let app = test_app(&server).await;

    put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 0.5}}}),
    )
    .await;
    put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 1.5}}}),
    )
    .await;

    let resp = app
        .oneshot(req_get(&format!("/api/assets/{id}/edits/history")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("history status {}", resp.status());
    }
    let entries: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
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
    mock_asset_metadata(&server, id, "abc").await;
    let app = test_app(&server).await;

    let first = put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 0.5}}}),
    )
    .await;
    let first_hash = first["hash"].as_str().unwrap().to_string();
    put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 1.5}}}),
    )
    .await;

    let resp = app
        .clone()
        .oneshot(req_get(&format!("/api/assets/{id}/edits/history")))
        .await
        .unwrap();
    let history: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let entry_id = history
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .find(|e| e["manifest_hash"] == serde_json::Value::String(first_hash.clone()))
        })
        .and_then(|e| e["id"].as_i64())
        .expect("history entry id");

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/edits/restore"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"entry_id": entry_id}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if restored["manifest"]["ops"]["exposure"]["ev"] != 0.5 {
        panic!("restore body: {restored}");
    }

    let resp = app
        .oneshot(req_get(&format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    let current: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if current["manifest"]["ops"]["exposure"]["ev"] != 0.5 {
        panic!("get-after-restore: {current}");
    }
}

#[tokio::test]
async fn restore_after_reset_refreshes_upstream_meta() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_metadata(&server, id, "abc").await;
    let app = test_app(&server).await;

    let saved = put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 0.5}}}),
    )
    .await;
    let hash = saved["hash"].as_str().unwrap().to_string();

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
        .clone()
        .oneshot(req_get(&format!("/api/assets/{id}/edits/history")))
        .await
        .unwrap();
    let history: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let entry_id = history
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .find(|e| e["manifest_hash"] == serde_json::Value::String(hash.clone()))
        })
        .and_then(|e| e["id"].as_i64())
        .expect("history entry id");

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/edits/restore"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"entry_id": entry_id}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if restored["immich_checksum"] != "abc" || restored["immich_updated_at"].is_null() {
        panic!("restore meta: {restored}");
    }
}

#[tokio::test]
async fn persisted_preview_revalidates_with_etag() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    mock_asset_metadata(&server, id, "abc").await;
    let app = test_app(&server).await;

    let uri = format!("/api/assets/{id}/preview?max=512");
    let resp = app.clone().oneshot(req_get(&uri)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("first status {}", resp.status());
    }
    let etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .expect("etag on first render");
    let cache_control = resp
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !cache_control.contains("must-revalidate") {
        panic!("cache-control: {cache_control}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&uri)
                .header("if-none-match", &etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_MODIFIED {
        panic!("expected 304, got {}", resp.status());
    }

    put_edits(
        &app,
        id,
        serde_json::json!({"schema_version": 2, "ops": {"exposure": {"ev": 1.5}}}),
    )
    .await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(&uri)
                .header("if-none-match", &etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("edited preview should re-render, got {}", resp.status());
    }
    let next = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if next == etag {
        panic!("etag unchanged after edit: {next}");
    }
}

fn jpeg_with_camera_and_gps() -> Vec<u8> {
    use little_exif::exif_tag::ExifTag;
    let rgb = vec![128u8; 64 * 48 * 3];
    let mut jpeg = raw_pipeline::encode::encode_jpeg_rgb(
        raw_pipeline::encode::ImageRgb8 {
            rgb: &rgb,
            width: 64,
            height: 48,
        },
        90,
        raw_pipeline::frame::JpegSubsampling::Chroma420,
        raw_pipeline::frame::OutputColorSpace::SRgb,
    )
    .unwrap();
    let mut meta = little_exif::metadata::Metadata::new();
    meta.set_tag(ExifTag::Make("SONY".into()));
    meta.set_tag(ExifTag::GPSLatitudeRef("N".into()));
    meta.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
    raw_pipeline::exif::inject(&mut jpeg, &meta, little_exif::filetype::FileExtension::JPEG)
        .unwrap();
    jpeg
}

#[tokio::test]
async fn export_metadata_keeps_or_strips_camera_and_location() {
    use little_exif::exif_tag::ExifTag;
    use little_exif::ifd::ExifTagGroup;
    for (label, options, want_make, want_gps) in [
        ("all", serde_json::json!({ "metadata": "all" }), true, true),
        (
            "no-location",
            serde_json::json!({ "metadata": "no-location" }),
            true,
            false,
        ),
        (
            "none",
            serde_json::json!({ "metadata": "none" }),
            false,
            false,
        ),
        (
            "legacy off",
            serde_json::json!({ "include_exif": false }),
            false,
            false,
        ),
    ] {
        let metadata = label;
        let server = MockServer::start().await;
        let id = asset_id();
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{id}/original")))
            .and(header("x-api-key", "test-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/jpeg")
                    .set_body_bytes(jpeg_with_camera_and_gps()),
            )
            .mount(&server)
            .await;
        mock_asset_detail(&server).await;
        let app = test_app(&server).await;
        let mut body = options;
        body["edits"] = serde_json::json!({});
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
            panic!("{metadata}: status {}", resp.status());
        }
        let bytes = body_bytes(resp).await;
        let parsed = raw_pipeline::exif::parse(&bytes);
        let has_make = parsed.as_ref().is_some_and(|m| {
            m.into_iter()
                .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"))
        });
        let has_gps = parsed
            .as_ref()
            .is_some_and(|m| m.into_iter().any(|t| t.get_group() == ExifTagGroup::GPS));
        if has_make != want_make || has_gps != want_gps {
            panic!("{metadata}: make {has_make}, gps {has_gps}");
        }
    }
}
