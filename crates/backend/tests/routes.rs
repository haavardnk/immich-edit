mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::MockServer;

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

#[tokio::test]
async fn health_returns_ok_with_redacted_config() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let bytes = body_bytes(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let s = json.to_string();
    if s.contains("test-key") {
        panic!("api key leaked: {s}");
    }
    if json["renderer_mode"] != "cpu" {
        panic!("renderer_mode field");
    }
    if json["renderer_active"] != "cpu" {
        panic!("renderer_active field");
    }
    if json["immich_reachable"] != true {
        panic!("ping flag");
    }
    if json["config"]["immich_api_key_present"] != true {
        panic!("redacted flag");
    }
}

#[tokio::test]
async fn lists_albums() {
    let server = MockServer::start().await;
    mock_albums(&server).await;
    let app = router(test_state(&server).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/albums")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["albumName"] != "Test Album" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn album_detail_returns_assets() {
    let server = MockServer::start().await;
    mock_album_detail(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/albums/{}", album_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["assets"][0]["originalFileName"] != "DSC0001.ARW" {
        panic!("asset: {json}");
    }
}

#[tokio::test]
async fn asset_thumb_proxies_bytes_and_content_type() {
    let server = MockServer::start().await;
    mock_thumb(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/thumb?size=preview", asset_id()))
                .body(Body::empty())
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
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg soi");
    }
}

#[tokio::test]
async fn asset_thumb_rejects_bad_size() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/thumb?size=nope", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn unknown_api_returns_json_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/does/not/exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !ct.contains("application/json") {
        panic!("content-type: {ct}");
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["code"] != "not_found" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn unknown_non_api_returns_plain_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/something")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if ct.contains("application/json") {
        panic!("non-api should not be JSON 404: {ct}");
    }
}

#[tokio::test]
async fn upstream_404_maps_to_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn asset_detail_returns_exif_and_favorite() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["isFavorite"] != true {
        panic!("favorite: {json}");
    }
    if json["exifInfo"]["rating"] != 4 {
        panic!("rating: {json}");
    }
    if json["exifInfo"]["exifImageWidth"] != 4032 {
        panic!("width: {json}");
    }
    if json["tags"][0]["value"] != "Landscape" {
        panic!("tags: {json}");
    }
}

#[tokio::test]
async fn asset_update_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_asset_update(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{}", asset_id()))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"rating":5,"isFavorite":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["exifInfo"]["rating"] != 5 {
        panic!("rating: {json}");
    }
}

#[tokio::test]
async fn tags_upsert_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_tag_upsert(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/tags")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"tags":["New"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["value"] != "New" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn tag_asset_add_and_remove_proxy() {
    let server = MockServer::start().await;
    mock_tag_asset(&server).await;
    mock_untag_asset(&server).await;
    let app = router(test_state(&server).await);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/tags/{}/assets/{}", tag_id(), asset_id()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["success"] != true {
        panic!("body: {json}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/tags/{}/assets/{}", tag_id(), asset_id()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("delete status {}", resp.status());
    }
}
