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

#[tokio::test]
async fn create_copy_seeds_edits_and_lists_under_the_master() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let id = asset_id();
    let app = test_app(&server).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"schema_version":2,"ops":{"exposure":{"ev":1.5}}}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put edits status {}", resp.status());
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/copies"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Warm"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::CREATED {
        panic!("create status {}", resp.status());
    }
    let created = body_json(resp).await;
    if created["id"] != format!("{id}_1") {
        panic!("copy id: {created}");
    }
    if created["name"] != "Warm" {
        panic!("copy name: {created}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}_1/edits"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let seeded = body_json(resp).await;
    if seeded["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("copy did not inherit edits: {seeded}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}_1/copies"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let listed = body_json(resp).await;
    if listed.as_array().map(Vec::len) != Some(1) {
        panic!("listing a copy should show its siblings: {listed}");
    }
}

#[tokio::test]
async fn neutral_copy_starts_empty_and_delete_leaves_the_master() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let id = asset_id();
    let app = test_app(&server).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{id}/edits"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"schema_version":2,"ops":{"exposure":{"ev":1.5}}}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put edits status {}", resp.status());
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/copies"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"from":"neutral"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::CREATED {
        panic!("create status {}", resp.status());
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}_1/edits"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let seeded = body_json(resp).await;
    if !seeded["manifest"]["ops"]
        .as_object()
        .map(|m| m.is_empty())
        .unwrap_or(false)
    {
        panic!("neutral copy inherited edits: {seeded}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/copies/{id}_1"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Mono"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let renamed = body_json(resp).await;
    if renamed["name"] != "Mono" {
        panic!("rename: {renamed}");
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/copies/{id}_1"))
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
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/edits"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let master = body_json(resp).await;
    if master["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("master edits were collateral damage: {master}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/copies/{id}_1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("second delete status {}", resp.status());
    }
}

#[tokio::test]
async fn rejects_a_copy_id_with_a_bad_index() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}_0/edits", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() == StatusCode::OK {
        panic!("_0 should not parse as a copy key");
    }
}

#[tokio::test]
async fn album_listing_places_copies_after_their_master() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    mock_album_detail(&server).await;
    let id = asset_id();
    let app = test_app(&server).await;

    for name in ["Warm", "Mono"] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/assets/{id}/copies"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        if resp.status() != StatusCode::CREATED {
            panic!("create status {}", resp.status());
        }
    }

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/albums/{}", album_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let album = body_json(resp).await;
    let assets = album["assets"].as_array().cloned().unwrap_or_default();
    let ids: Vec<&str> = assets.iter().filter_map(|a| a["id"].as_str()).collect();
    if ids != [id.to_string(), format!("{id}_1"), format!("{id}_2")] {
        panic!("expansion order: {ids:?}");
    }
    if assets[1]["copyOf"] != id.to_string() || assets[1]["copyLabel"] != "Warm" {
        panic!("copy metadata: {}", assets[1]);
    }
    if assets[0].get("copyOf").is_some() {
        panic!("master should not carry copyOf: {}", assets[0]);
    }
    if assets[2]["originalFileName"] != "DSC0001.ARW" {
        panic!("copies should inherit master metadata: {}", assets[2]);
    }
}
