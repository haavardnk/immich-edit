mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use raw_pipeline::frame::{BitDepth, OutputColorSpace, OutputFormat, PngCompression, RawFrame};
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

fn magenta_png() -> Vec<u8> {
    raw_pipeline::encode::encode_from_rgb8(
        &[255, 0, 255].repeat(100),
        10,
        10,
        &OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression: PngCompression::Fast,
        },
        OutputColorSpace::SRgb,
    )
    .unwrap()
}

async fn mock_arw_original(server: &MockServer, id: uuid::Uuid) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(std::fs::read(fixture).expect("committed Sony ARW fixture")),
        )
        .mount(server)
        .await;
}

async fn send(app: &Router, method: &str, uri: &str, body: Body) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn import(app: &Router, bytes: Vec<u8>) -> axum::response::Response {
    send(app, "POST", "/api/watermarks?name=logo", Body::from(bytes)).await
}

async fn json(resp: axum::response::Response) -> serde_json::Value {
    serde_json::from_slice(&body_bytes(resp).await).unwrap()
}

async fn export(app: &Router, body: serde_json::Value) -> axum::response::Response {
    let uri = format!("/api/assets/{}/export", asset_id());
    send(app, "POST", &uri, Body::from(body.to_string())).await
}

fn rgb_at(frame: &RawFrame, x: usize, y: usize) -> Vec<f32> {
    let i = (y * frame.meta.width + x) * frame.cpp;
    frame.data[i..i + 3].to_vec()
}

#[tokio::test]
async fn the_watermark_library_round_trips_the_png() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let png = magenta_png();

    let created = import(&app, png.clone()).await;
    if created.status() != StatusCode::CREATED {
        panic!("import: {}", created.status());
    }
    let meta = json(created).await;
    if (meta["width"].as_u64(), meta["height"].as_u64()) != (Some(10), Some(10)) {
        panic!("meta: {meta}");
    }
    let id = meta["id"].as_str().unwrap().to_string();

    let listed = json(send(&app, "GET", "/api/watermarks", Body::empty()).await).await;
    if listed[0]["id"] != id.as_str() {
        panic!("list: {listed}");
    }
    let served = send(
        &app,
        "GET",
        &format!("/api/watermarks/{id}/png"),
        Body::empty(),
    )
    .await;
    let content_type = served.headers()["content-type"]
        .to_str()
        .unwrap()
        .to_string();
    let cache = served.headers()["cache-control"]
        .to_str()
        .unwrap()
        .to_string();
    if content_type != "image/png" || !cache.contains("immutable") {
        panic!("png headers: {content_type} {cache}");
    }
    if body_bytes(served).await != png {
        panic!("png bytes differ from the import");
    }

    let duplicate = import(&app, png).await;
    if duplicate.status() != StatusCode::CONFLICT {
        panic!("duplicate: {}", duplicate.status());
    }
    let invalid = import(&app, b"not a png".to_vec()).await;
    if invalid.status() != StatusCode::BAD_REQUEST {
        panic!("invalid: {}", invalid.status());
    }

    let deleted = send(
        &app,
        "DELETE",
        &format!("/api/watermarks/{id}"),
        Body::empty(),
    )
    .await;
    if deleted.status() != StatusCode::NO_CONTENT {
        panic!("delete: {}", deleted.status());
    }
    let listed = json(send(&app, "GET", "/api/watermarks", Body::empty()).await).await;
    if listed.as_array().is_none_or(|a| !a.is_empty()) {
        panic!("list after delete: {listed}");
    }
    let kept = send(
        &app,
        "GET",
        &format!("/api/watermarks/{id}/png"),
        Body::empty(),
    )
    .await;
    if kept.status() != StatusCode::OK {
        panic!(
            "a deleted watermark must stay readable for queued jobs: {}",
            kept.status()
        );
    }
}

#[tokio::test]
async fn export_composites_the_watermark_in_its_corner() {
    let server = MockServer::start().await;
    mock_arw_original(&server, asset_id()).await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let id = json(import(&app, magenta_png()).await).await["id"].clone();

    let base = serde_json::json!({
        "edits": {},
        "format": "png",
        "resize_mode": "dimensions",
        "resize_width": 600,
        "resize_height": 600
    });
    let plain =
        raw_pipeline::decode::decode(&body_bytes(export(&app, base.clone()).await).await).unwrap();
    let mut body = base;
    body["watermark_id"] = id;
    body["watermark_size"] = 0.25.into();
    body["watermark_opacity"] = 1.0.into();
    let resp = export(&app, body).await;
    if resp.status() != StatusCode::OK {
        panic!("watermarked export: {}", resp.status());
    }
    let marked = raw_pipeline::decode::decode(&body_bytes(resp).await).unwrap();

    let width = marked.meta.width;
    let height = marked.meta.height;
    let short = width.min(height) as f32;
    let edge = (short * 0.25).round() as usize;
    let inset = (short * 0.03).round() as usize;
    let centre = rgb_at(&marked, width - inset - edge / 2, height - inset - edge / 2);
    if centre
        .iter()
        .zip([1.0, 0.0, 1.0])
        .any(|(v, want)| (v - want).abs() > 0.02)
    {
        panic!("watermark centre is {centre:?}");
    }
    if rgb_at(&marked, inset, inset) != rgb_at(&plain, inset, inset) {
        panic!("the watermark touched the opposite corner");
    }
}

#[tokio::test]
async fn export_rejects_an_unknown_watermark() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = export(
        &app,
        serde_json::json!({ "edits": {}, "watermark_id": "missing" }),
    )
    .await;
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}
