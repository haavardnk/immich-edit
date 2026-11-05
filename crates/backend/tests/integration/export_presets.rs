use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use wiremock::MockServer;

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
async fn an_export_preset_keeps_its_destination() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let create = |body: serde_json::Value| {
        let app = app.clone();
        async move {
            let resp = app
                .oneshot(json_request("POST", "/api/export-presets", body))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            body_json(resp).await
        }
    };

    let download = create(serde_json::json!({ "name": "Web", "form": {} })).await;
    let immich = create(serde_json::json!({
        "name": "Album",
        "form": {},
        "destination": "immich"
    }))
    .await;
    assert_eq!(download["destination"], "download");
    assert_eq!(immich["destination"], "immich");

    let id = immich["id"].as_str().unwrap();
    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/export-presets/{id}"),
            serde_json::json!({ "name": "Album 2", "form": { "format": "png" } }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["destination"], "immich");

    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/export-presets/{id}"),
            serde_json::json!({ "name": "Album 2", "form": {}, "destination": "download" }),
        ))
        .await
        .unwrap();
    assert_eq!(body_json(resp).await["destination"], "download");

    let listed = body_json(app.clone().oneshot(list_request()).await.unwrap()).await;
    let destinations: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|preset| preset["destination"].as_str())
        .collect();
    assert_eq!(destinations, ["download", "download"]);

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/export-presets",
            serde_json::json!({ "name": "Bad", "form": {}, "destination": "dropbox" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
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
