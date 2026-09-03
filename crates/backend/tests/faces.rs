mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn mock_faces(server: &MockServer, status: u16, body: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path("/api/faces"))
        .and(query_param("id", asset_id().to_string()))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(server)
        .await;
}

async fn get_faces(server: &MockServer) -> serde_json::Value {
    let app = test_app(server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/faces", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    body_json(resp).await
}

#[tokio::test]
async fn faces_are_normalized_against_the_reported_image_size() {
    let server = MockServer::start().await;
    mock_faces(
        &server,
        200,
        serde_json::json!([
            {
                "id": "11111111-0000-0000-0000-000000000001",
                "imageWidth": 1000,
                "imageHeight": 500,
                "boundingBoxX1": 100,
                "boundingBoxY1": 50,
                "boundingBoxX2": 300,
                "boundingBoxY2": 150
            },
            {
                "id": "11111111-0000-0000-0000-000000000002",
                "imageWidth": 1000,
                "imageHeight": 500,
                "boundingBoxX1": 900,
                "boundingBoxY1": 400,
                "boundingBoxX2": 700,
                "boundingBoxY2": 200
            }
        ]),
    )
    .await;

    let faces = get_faces(&server).await;
    let list = faces.as_array().unwrap();
    if list.len() != 2 {
        panic!("faces: {faces}");
    }
    if list[0]["x"] != 0.1 || list[0]["y"] != 0.1 || list[0]["w"] != 0.2 || list[0]["h"] != 0.2 {
        panic!("first face: {}", list[0]);
    }
    if list[1]["x"] != 0.7 || list[1]["y"] != 0.4 || list[1]["w"] != 0.2 || list[1]["h"] != 0.4 {
        panic!("second face: {}", list[1]);
    }
    if list[0]["source_w"] != 1000 || list[0]["source_h"] != 500 {
        panic!("source dims: {}", list[0]);
    }
}

#[tokio::test]
async fn degenerate_and_unsized_faces_are_dropped() {
    let server = MockServer::start().await;
    mock_faces(
        &server,
        200,
        serde_json::json!([
            {
                "id": "11111111-0000-0000-0000-000000000003",
                "imageWidth": 0,
                "imageHeight": 0,
                "boundingBoxX1": 0,
                "boundingBoxY1": 0,
                "boundingBoxX2": 10,
                "boundingBoxY2": 10
            },
            {
                "id": "11111111-0000-0000-0000-000000000004",
                "imageWidth": 1000,
                "imageHeight": 500,
                "boundingBoxX1": 200,
                "boundingBoxY1": 100,
                "boundingBoxX2": 200,
                "boundingBoxY2": 300
            }
        ]),
    )
    .await;

    let faces = get_faces(&server).await;
    if !faces.as_array().unwrap().is_empty() {
        panic!("faces: {faces}");
    }
}

#[tokio::test]
async fn missing_upstream_face_endpoint_degrades_to_empty() {
    let server = MockServer::start().await;
    mock_faces(&server, 404, serde_json::json!({ "message": "Not found" })).await;

    let faces = get_faces(&server).await;
    if !faces.as_array().unwrap().is_empty() {
        panic!("faces: {faces}");
    }
}
