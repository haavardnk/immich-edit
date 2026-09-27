use crate::common::*;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;
use wiremock::MockServer;

fn luma_dc_quantizer(jpeg: &[u8]) -> u8 {
    let mut at = 2;
    while jpeg[at + 1] != 0xdb {
        at += 2 + usize::from(u16::from_be_bytes([jpeg[at + 2], jpeg[at + 3]]));
    }
    jpeg[at + 5]
}

fn revalidate(uri: &str, etag: &str) -> Request<Body> {
    let mut req = get(uri);
    req.headers_mut()
        .insert("if-none-match", HeaderValue::from_str(etag).unwrap());
    req
}

#[tokio::test]
async fn live_preview_renders_jpeg_and_returns_meta_id() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    let app = test_app(&server).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            json!({"max_edge": 512, "edits": {"basic": {"exposure_ev": 1.0}}}),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let ct = header_str(&resp, "content-type").unwrap_or_default();
    if !ct.starts_with("image/jpeg") {
        panic!("content-type: {ct}");
    }
    let Some(meta_id) = header_str(&resp, "x-preview-meta-id") else {
        panic!("missing meta header");
    };
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg");
    }

    let resp = app
        .oneshot(get(&format!("/api/assets/{id}/preview/meta/{meta_id}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("meta status {}", resp.status());
    }
    let meta = body_json(resp).await;
    if meta["width"].as_u64().unwrap_or(0) == 0 {
        panic!("meta dims: {meta}");
    }
    let bins = meta["histogram"]["l"].as_array().unwrap();
    if bins.len() != 256 {
        panic!("histogram bins: {}", bins.len());
    }
}

#[tokio::test]
async fn live_previews_encode_lighter_than_persisted_ones() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    let app = test_app(&server).await;

    let persisted = app
        .clone()
        .oneshot(get(&format!("/api/assets/{id}/preview?max=512")))
        .await
        .unwrap();
    let persisted = body_bytes(persisted).await;
    let live = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            json!({"max_edge": 512}),
        ))
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
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{}/preview", asset_id()),
            json!({"max_edge": 10}),
        ))
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
    mock_original(&server, id).await;
    let app = test_app(&server).await;

    let resp = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            json!({"max_edge": 512, "clip_warn": true}),
        ))
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
    mock_original(&server, id).await;
    let app = test_app(&server).await;

    let etag_for = async |clip: bool| {
        let resp = app
            .clone()
            .oneshot(get(&format!(
                "/api/assets/{id}/preview?max=512&clip={clip}"
            )))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("status {}", resp.status());
        }
        header_str(&resp, "etag").expect("etag")
    };
    let plain = etag_for(false).await;
    let clipped = etag_for(true).await;
    if plain == clipped {
        panic!("clip flag must change the etag: {plain}");
    }

    let resp = app
        .oneshot(revalidate(
            &format!("/api/assets/{id}/preview?max=512&clip=true"),
            &plain,
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("stale etag must not revalidate: {}", resp.status());
    }
}

#[tokio::test]
async fn persisted_preview_revalidates_with_etag() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let uri = format!("/api/assets/{id}/preview?max=512");
    let resp = app.clone().oneshot(get(&uri)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("first status {}", resp.status());
    }
    let etag = header_str(&resp, "etag").expect("etag on first render");
    let cache_control = header_str(&resp, "cache-control").unwrap_or_default();
    if !cache_control.contains("must-revalidate") {
        panic!("cache-control: {cache_control}");
    }

    let resp = app.clone().oneshot(revalidate(&uri, &etag)).await.unwrap();
    if resp.status() != StatusCode::NOT_MODIFIED {
        panic!("expected 304, got {}", resp.status());
    }

    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            json!({"schema_version": 2, "ops": {"exposure": {"ev": 1.5}}}),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put: {}", resp.status());
    }

    let resp = app.oneshot(revalidate(&uri, &etag)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("edited preview should re-render, got {}", resp.status());
    }
    let next = header_str(&resp, "etag").unwrap_or_default();
    if next == etag {
        panic!("etag unchanged after edit: {next}");
    }
}
