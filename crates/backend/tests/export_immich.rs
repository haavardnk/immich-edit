mod common;

use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::routes::export::{
    ColorSpaceOpt, ExportParams, ExportToImmichBody, StackPrimary, hash_request, resolve_filename,
};

#[test]
fn hash_request_differs_by_color_space() {
    let asset = AssetKey::master(uuid::Uuid::nil());
    let make = |cs: ColorSpaceOpt| ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams {
            color_space: cs,
            ..ExportParams::default()
        },
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
    };
    let srgb = hash_request(asset, &make(ColorSpaceOpt::Srgb));
    let p3 = hash_request(asset, &make(ColorSpaceOpt::Displayp3));
    assert_ne!(srgb, p3);
}

#[test]
fn hash_request_differs_by_filename_template() {
    let asset = AssetKey::master(uuid::Uuid::nil());
    let make = |template: Option<&str>| ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams {
            filename_template: template.map(str::to_string),
            ..ExportParams::default()
        },
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
    };
    assert_ne!(
        hash_request(asset, &make(Some("{name}_a"))),
        hash_request(asset, &make(Some("{name}_b")))
    );
}

#[test]
fn resolves_with_no_existing() {
    let name = resolve_filename("DSC0001_edit", "jpg", &["DSC0001.ARW".into()]);
    assert_eq!(name, "DSC0001_edit.jpg");
}

#[test]
fn resolves_increments_on_collision() {
    let existing = vec!["DSC0001.ARW".into(), "DSC0001_edit.jpg".into()];
    let name = resolve_filename("DSC0001_edit", "jpg", &existing);
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
    let name = resolve_filename("DSC0001_edit", "jpg", &existing);
    assert_eq!(name, "DSC0001_edit_4.jpg");
}

#[test]
fn resolves_case_insensitive() {
    let existing = vec!["IMG.JPG".into(), "IMG_EDIT.JPG".into()];
    let name = resolve_filename("IMG_edit", "jpg", &existing);
    assert_eq!(name, "IMG_edit_2.jpg");
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use common::*;
use http_body_util::BodyExt;
use immich_edit_backend::immich::ImmichClient;
use immich_edit_backend::immich::client::{ImmichAuth, UploadRequest};
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
    let asset = AssetKey::master(asset_id());
    let uploaded = Uuid::new_v4();
    let body = ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams::default(),
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
    };
    let hash = hash_request(asset, &body);
    state
        .edits
        .put_export_job_uploaded(
            test_user_id(),
            asset,
            "key-1",
            &hash,
            uploaded,
            "x_edit.jpg",
            "created",
        )
        .await
        .unwrap();
    state
        .edits
        .complete_export_job(test_user_id(), asset, "key-1", &[])
        .await
        .unwrap();

    let app = seed_and_wrap(&server, state).await;
    let req_body = serde_json::json!({});
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

fn test_client(server: &MockServer) -> ImmichClient {
    ImmichClient::with_auth(
        server.uri().parse().unwrap(),
        ImmichAuth::ApiKey(TEST_API_KEY.into()),
        Duration::from_secs(5),
    )
    .unwrap()
}

#[tokio::test]
async fn upload_asset_sends_only_supported_multipart_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/assets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": Uuid::new_v4(),
            "status": "created"
        })))
        .mount(&server)
        .await;

    test_client(&server)
        .upload_asset(UploadRequest {
            filename: "DSC0001_edit.jpg",
            content_type: "image/jpeg",
            bytes: Bytes::from_static(&[0xFF, 0xD8]),
            is_favorite: true,
            created_at: "2026-01-01T00:00:00Z",
            modified_at: "2026-01-01T00:00:00Z",
        })
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body).into_owned();
    for field in [
        "assetData",
        "filename",
        "fileCreatedAt",
        "fileModifiedAt",
        "isFavorite",
    ] {
        if !body.contains(&format!("name=\"{field}\"")) {
            panic!("upload is missing required field {field}");
        }
    }
    for field in ["deviceId", "deviceAssetId"] {
        if body.contains(&format!("name=\"{field}\"")) {
            panic!("upload still sends removed field {field}");
        }
    }
}

#[tokio::test]
async fn stack_primary_update_uses_put() {
    let server = MockServer::start().await;
    let stack = Uuid::new_v4();
    let primary = asset_id();
    Mock::given(method("PUT"))
        .and(path(format!("/api/stacks/{stack}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": stack,
            "primaryAssetId": primary,
            "assets": []
        })))
        .mount(&server)
        .await;

    let updated = test_client(&server)
        .update_stack_primary(stack, primary)
        .await
        .unwrap();
    if updated.primary_asset_id != primary {
        panic!("primary: {}", updated.primary_asset_id);
    }
}
