use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

mod lifecycle;
mod restore;

fn snapshots_uri(key: &str) -> String {
    format!("/api/assets/{key}/snapshots")
}

fn snapshot_uri(key: &str, snapshot_id: &str) -> String {
    format!("/api/assets/{key}/snapshots/{snapshot_id}")
}

async fn send(app: &axum::Router, req: Request<Body>) -> Response {
    app.clone().oneshot(req).await.unwrap()
}

async fn create(app: &axum::Router, key: &str, name: &str, manifest: Value) -> Value {
    let resp = send(
        app,
        json_request(
            "POST",
            &snapshots_uri(key),
            json!({ "name": name, "manifest": manifest }),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

async fn list(app: &axum::Router, key: &str) -> Vec<Value> {
    let resp = send(app, get(&snapshots_uri(key))).await;
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await.as_array().cloned().unwrap()
}

fn names(snapshots: &[Value]) -> Vec<String> {
    snapshots
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

fn update_request(key: &str, snapshot_id: &str, body: Value) -> Request<Body> {
    json_request("PATCH", &snapshot_uri(key, snapshot_id), body)
}

fn restore_request(key: &str, snapshot_id: &str, if_match: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("{}/restore", snapshot_uri(key, snapshot_id)));
    if let Some(tag) = if_match {
        builder = builder.header("if-match", format!("\"{tag}\""));
    }
    builder.body(Body::empty()).unwrap()
}

fn id_of(snapshot: &Value) -> String {
    snapshot["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_snapshot_survives_create_list_rename_and_delete() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();

    let first = create(&app, &key, "  First  ", exposure_manifest(0.5)).await;
    assert_eq!(first["name"], "First");
    assert_eq!(first["manifest"]["ops"]["exposure"]["ev"], 0.5);
    assert_eq!(first["created_at"], first["updated_at"]);
    assert!(!first["manifest_hash"].as_str().unwrap().is_empty());
    create(&app, &key, "Second", exposure_manifest(1.0)).await;
    assert_eq!(names(&list(&app, &key).await), ["Second", "First"]);

    let first_id = id_of(&first);
    let resp = send(
        &app,
        update_request(&key, &first_id, json!({ "name": " Renamed " })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let renamed = body_json(resp).await;
    assert_eq!(renamed["id"], first["id"]);
    assert_eq!(renamed["name"], "Renamed");
    assert_eq!(renamed["created_at"], first["created_at"]);
    assert_eq!(renamed["manifest"], first["manifest"]);
    assert_eq!(names(&list(&app, &key).await), ["Second", "Renamed"]);

    let delete = || empty_request("DELETE", &snapshot_uri(&key, &first_id));
    assert_eq!(send(&app, delete()).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(names(&list(&app, &key).await), ["Second"]);
    assert_eq!(send(&app, delete()).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn updating_with_current_edits_replaces_the_manifest() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let first = create(&app, &key, "Look", exposure_manifest(0.5)).await;
    let first_id = id_of(&first);

    let resp = send(
        &app,
        update_request(
            &key,
            &first_id,
            json!({ "manifest": exposure_manifest(1.5) }),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["id"], first["id"]);
    assert_eq!(updated["name"], "Look");
    assert_eq!(updated["created_at"], first["created_at"]);
    assert_eq!(updated["manifest"]["ops"]["exposure"]["ev"], 1.5);
    assert_ne!(updated["manifest_hash"], first["manifest_hash"]);

    let resp = send(
        &app,
        update_request(
            &key,
            &first_id,
            json!({ "name": " Both ", "manifest": exposure_manifest(-1.0) }),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let both = body_json(resp).await;
    assert_eq!(both["name"], "Both");
    assert_eq!(both["manifest"]["ops"]["exposure"]["ev"], -1.0);
    assert_eq!(list(&app, &key).await, [both]);
}

#[tokio::test]
async fn an_empty_update_is_a_bad_request() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let snapshot = create(&app, &key, "Kept", exposure_manifest(0.5)).await;

    let resp = send(&app, update_request(&key, &id_of(&snapshot), json!({}))).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["code"], "bad_request");
    assert_eq!(
        body["message"],
        "snapshot update needs a name or a manifest"
    );
    assert_eq!(list(&app, &key).await, [snapshot]);
}

#[tokio::test]
async fn an_unknown_snapshot_is_not_found() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let unknown = Uuid::new_v4().to_string();
    for req in [
        restore_request(&key, &unknown, None),
        update_request(&key, &unknown, json!({ "name": "Gone" })),
        update_request(
            &key,
            &unknown,
            json!({ "manifest": exposure_manifest(1.0) }),
        ),
        empty_request("DELETE", &snapshot_uri(&key, &unknown)),
    ] {
        assert_eq!(send(&app, req).await.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn a_snapshot_is_not_reachable_through_another_asset() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let snapshot = create(&app, &key, "Mine", exposure_manifest(0.5)).await;
    let snapshot_id = id_of(&snapshot);

    for other in [Uuid::new_v4().to_string(), format!("{key}_1")] {
        assert!(list(&app, &other).await.is_empty(), "{other}");
        for req in [
            restore_request(&other, &snapshot_id, None),
            update_request(&other, &snapshot_id, json!({ "name": "Stolen" })),
            update_request(
                &other,
                &snapshot_id,
                json!({ "manifest": exposure_manifest(2.0) }),
            ),
            empty_request("DELETE", &snapshot_uri(&other, &snapshot_id)),
        ] {
            assert_eq!(send(&app, req).await.status(), StatusCode::NOT_FOUND);
        }
    }
    assert_eq!(names(&list(&app, &key).await), ["Mine"]);
}

#[tokio::test]
async fn snapshot_names_are_validated() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    create(&app, &key, "Kept", exposure_manifest(0.5)).await;
    let long = "x".repeat(81);
    let unknown = Uuid::new_v4().to_string();
    for (req, message) in [
        (
            json_request(
                "POST",
                &snapshots_uri(&key),
                json!({ "name": "  ", "manifest": exposure_manifest(1.0) }),
            ),
            "snapshot name required",
        ),
        (
            update_request(&key, &unknown, json!({ "name": long })),
            "snapshot name longer than 80 characters",
        ),
    ] {
        let resp = send(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{message}");
        let body = body_json(resp).await;
        assert_eq!(body["code"], "bad_request");
        assert_eq!(body["message"], message);
    }
    assert_eq!(names(&list(&app, &key).await), ["Kept"]);
}

#[tokio::test]
async fn an_unknown_field_is_unprocessable() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let snapshot = create(&app, &key, "Kept", exposure_manifest(0.5)).await;
    for req in [
        json_request(
            "POST",
            &snapshots_uri(&key),
            json!({ "name": "Extra", "manifest": exposure_manifest(1.0), "pinned": true }),
        ),
        update_request(
            &key,
            &id_of(&snapshot),
            json!({ "name": "Extra", "pinned": true }),
        ),
    ] {
        assert_eq!(
            send(&app, req).await.status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(names(&list(&app, &key).await), ["Kept"]);
}
