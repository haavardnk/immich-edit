mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::MockServer;

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn list_request() -> Request<Body> {
    Request::builder()
        .uri("/api/export-presets")
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn an_export_preset_survives_create_list_update_and_delete() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let form = serde_json::json!({ "format": "jpeg", "quality": 85, "albumIds": ["a1"] });
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/export-presets",
            serde_json::json!({ "name": "  Web JPEG  ", "form": form }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created = body_json(resp).await;
    assert_eq!(created["name"], "Web JPEG");
    assert_eq!(created["form"], form);
    let id = created["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/export-presets/{id}"),
            serde_json::json!({ "name": "Print TIFF", "form": { "format": "tiff" } }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let listed = body_json(app.clone().oneshot(list_request()).await.unwrap()).await;
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["name"], "Print TIFF");
    assert_eq!(listed[0]["form"], serde_json::json!({ "format": "tiff" }));

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/export-presets/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let listed = body_json(app.oneshot(list_request()).await.unwrap()).await;
    assert_eq!(listed.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn export_preset_bodies_are_validated() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for body in [
        serde_json::json!({ "name": "  ", "form": {} }),
        serde_json::json!({ "name": "x".repeat(81), "form": {} }),
        serde_json::json!({ "name": "List", "form": [1, 2] }),
        serde_json::json!({ "name": "Huge", "form": { "blob": "x".repeat(17 * 1024) } }),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/export-presets", body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
    }
}
