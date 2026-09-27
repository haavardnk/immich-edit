mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use tower::ServiceExt;
use wiremock::MockServer;

fn upload(bytes: Vec<u8>) -> Request<Body> {
    Request::post("/api/rasters?width=4&height=3")
        .header("content-type", "application/octet-stream")
        .body(Body::from(bytes))
        .unwrap()
}

#[tokio::test]
async fn raster_upload_get_roundtrip() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let bytes = vec![0xABu8; 4 * 3];
    let resp = app.clone().oneshot(upload(bytes.clone())).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    let raster_id = v["raster_id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(get(&format!("/api/rasters/{raster_id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("x-raster-width").unwrap(), "4");
    assert_eq!(resp.headers().get("x-raster-height").unwrap(), "3");
    let got = body_bytes(resp).await;
    assert_eq!(got, bytes);
}

#[tokio::test]
async fn raster_upload_rejects_bad_size() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app.oneshot(upload(vec![0u8; 5])).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
