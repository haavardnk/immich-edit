mod common;

use immich_edit_backend::routes::export::{
    ExportParams, ExportToImmichBody, StackPrimary, hash_request, resolve_filename,
};

#[test]
fn resolves_with_no_existing() {
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &["DSC0001.ARW".into()]);
    assert_eq!(name, "DSC0001_edit.jpg");
}

#[test]
fn resolves_increments_on_collision() {
    let existing = vec!["DSC0001.ARW".into(), "DSC0001_edit.jpg".into()];
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &existing);
    assert_eq!(name, "DSC0001_edit_2.jpg");
}

#[test]
fn resolves_skips_multiple_collisions() {
    let existing = vec![
        "DSC0001.ARW".into(),
        "DSC0001_edit.jpg".into(),
        "DSC0001_edit_2.jpg".into(),
        "DSC0001_edit_3.jpg".into(),
    ];
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &existing);
    assert_eq!(name, "DSC0001_edit_4.jpg");
}

#[test]
fn resolves_case_insensitive() {
    let existing = vec!["IMG.JPG".into(), "IMG_EDIT.JPG".into()];
    let name = resolve_filename("IMG.JPG", "_edit", "jpg", &existing);
    assert_eq!(name, "IMG_edit_2.jpg");
}

#[test]
fn resolves_handles_no_extension_original() {
    let name = resolve_filename("raw", "_edit", "png", &["raw".into()]);
    assert_eq!(name, "raw_edit.png");
}

#[test]
fn resolves_custom_suffix() {
    let name = resolve_filename("a.arw", "-final", "tif", &["a.arw".into()]);
    assert_eq!(name, "a-final.tif");
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;
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
async fn export_immich_idempotency_returns_cached_without_reupload() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let asset = asset_id();
    let uploaded = Uuid::new_v4();
    let body = ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams::default(),
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
        filename_suffix: "_edit".into(),
    };
    let hash = hash_request(asset, &body);
    state
        .edits
        .put_export_job_uploaded(asset, "key-1", &hash, uploaded, "x_edit.jpg", "created")
        .await
        .unwrap();
    state
        .edits
        .complete_export_job(asset, "key-1", &[])
        .await
        .unwrap();

    let app = router(state);
    let req_body = serde_json::json!({"filename_suffix": "_edit"});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{asset}/export/immich"))
                .header("content-type", "application/json")
                .header("idempotency-key", "key-1")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["asset_id"].as_str() != Some(&uploaded.to_string()) {
        panic!("expected cached asset id: {json}");
    }
    if json["filename"] != "x_edit.jpg" {
        panic!("expected cached filename: {json}");
    }
    if json["status"] != "created" {
        panic!("expected cached status: {json}");
    }
}
