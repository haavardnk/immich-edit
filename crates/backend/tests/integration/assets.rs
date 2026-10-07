use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn asset_thumb_proxies_bytes_and_content_type() {
    for (query, upstream) in [
        ("?size=preview", "preview"),
        ("?size=thumbnail", "thumbnail"),
        ("", "preview"),
    ] {
        let server = MockServer::start().await;
        mock_thumb(&server, upstream).await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(get(&format!("/api/assets/{}/thumb{query}", asset_id())))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("{query}: status {}", resp.status());
        }
        let ct = header_str(&resp, "content-type").unwrap_or_default();
        if !ct.starts_with("image/jpeg") {
            panic!("{query}: content-type: {ct}");
        }
        let bytes = body_bytes(resp).await;
        if &bytes[..2] != b"\xff\xd8" {
            panic!("{query}: not jpeg soi");
        }
    }
}

#[tokio::test]
async fn asset_thumb_rejects_bad_size() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/assets/{}/thumb?size=nope", asset_id())))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn upstream_404_maps_to_404() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/assets/{}", asset_id())))
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
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/assets/{}", asset_id())))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
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
    if json["ownerId"] != test_user_id().to_string() {
        panic!("owner: {json}");
    }
}

#[tokio::test]
async fn asset_update_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_asset_update(&server).await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{}", asset_id()),
            json!({"rating": 5, "isFavorite": true}),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json["exifInfo"]["rating"] != 5 {
        panic!("rating: {json}");
    }
}

#[tokio::test]
async fn asset_update_reports_immich_refusal() {
    for (upstream, expected) in [
        (
            ResponseTemplate::new(400).set_body_json(json!({
                "message": "Not found or no asset.update access",
                "error": "Bad Request",
                "statusCode": 400
            })),
            "Immich refused the request: Not found or no asset.update access",
        ),
        (
            ResponseTemplate::new(400).set_body_json(
                json!({ "message": ["rating must be <= 5", "rating must be an integer"] }),
            ),
            "Immich refused the request: rating must be <= 5; rating must be an integer",
        ),
        (
            ResponseTemplate::new(413).set_body_string("<html>too large</html>"),
            "Immich refused the request: HTTP 413",
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path(format!("/api/assets/{}", asset_id())))
            .respond_with(upstream)
            .mount(&server)
            .await;
        let resp = test_app(&server)
            .await
            .oneshot(json_request(
                "PUT",
                &format!("/api/assets/{}", asset_id()),
                json!({"isFavorite": true}),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::UNPROCESSABLE_ENTITY {
            panic!("{expected}: status {}", resp.status());
        }
        let json = body_json(resp).await;
        if json["code"] != "upstream_rejected" || json["message"] != expected {
            panic!("body: {json}");
        }
    }
}
