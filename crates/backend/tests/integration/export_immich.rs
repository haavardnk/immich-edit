use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::routes::export::{
    ColorSpaceOpt, ExportParams, ExportToImmichBody, StackPrimary, hash_request, resolve_filename,
};
use immich_edit_backend::services::edits_store::ExportJobKey;

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

use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use immich_edit_backend::immich::ImmichClient;
use immich_edit_backend::immich::client::{ImmichAuth, UploadRequest};
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
    let job = ExportJobKey {
        owner: test_user_id(),
        asset_id: asset,
        key: "key-1",
    };
    state
        .edits
        .put_export_job_uploaded(job, &hash, uploaded, "x_edit.jpg", "created", &[])
        .await
        .unwrap();
    state.edits.complete_export_job(job, &[]).await.unwrap();

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
    let json = body_json(resp).await;
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

#[tokio::test]
async fn export_immich_resume_keeps_render_warnings() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let uploaded = Uuid::new_v4();
    mock_upload(&server, uploaded, 400).await;
    let state = test_state(&server).await;
    let asset = AssetKey::master(asset_id());
    let body = ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams::default(),
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
    };
    let job = ExportJobKey {
        owner: test_user_id(),
        asset_id: asset,
        key: "key-1",
    };
    let rendered = "Metadata not copied: no readable EXIF in the original";
    state
        .edits
        .put_export_job_uploaded(
            job,
            &hash_request(asset, &body),
            uploaded,
            "x_edit.jpg",
            "created",
            &[rendered.into()],
        )
        .await
        .unwrap();

    let resp = seed_and_wrap(&server, state)
        .await
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{asset}/export/immich"))
                .header("content-type", "application/json")
                .header("idempotency-key", "key-1")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    let warnings: Vec<&str> = json["warnings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|w| w.as_str())
        .collect();
    if warnings.len() != 2
        || warnings[0] != rendered
        || !warnings[1].starts_with("Metadata copy failed")
    {
        panic!("warnings {warnings:?}");
    }
    let requests = server.received_requests().await.unwrap();
    if requests
        .iter()
        .any(|r| r.method.as_str() == "POST" && r.url.path() == "/api/assets")
    {
        panic!("resumed job uploaded again");
    }
}

fn multipart_text<'a>(body: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("name=\"{field}\"\r\n\r\n");
    let start = body.find(&marker)? + marker.len();
    body[start..].split("\r\n").next()
}

#[tokio::test]
async fn export_immich_keeps_capture_metadata_and_date() {
    let jpeg = || {
        ResponseTemplate::new(200)
            .insert_header("content-type", "image/jpeg")
            .set_body_bytes(plain_jpeg())
    };
    for (label, original, metadata, capture, want_warning) in [
        (
            "rw2",
            raw_response("Panasonic_DMC-LX7_1-1.rw2", "image/x-panasonic-raw"),
            "all",
            Some("2023:11:18 09:49:12"),
            false,
        ),
        ("no exif", jpeg(), "all", None, true),
        ("no exif, none", jpeg(), "none", None, false),
    ] {
        let server = MockServer::start().await;
        mock_original_with(&server, asset_id(), original).await;
        mock_asset_detail(&server).await;
        mock_upload(&server, Uuid::new_v4(), 204).await;

        let resp = test_app(&server)
            .await
            .oneshot(json_request(
                "POST",
                &format!("/api/assets/{}/export/immich", asset_id()),
                serde_json::json!({
                    "edits": {},
                    "metadata": metadata,
                    "resize_mode": "dimensions",
                    "resize_width": 400
                }),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("{label}: status {}", resp.status());
        }
        let json = body_json(resp).await;
        let warned = json["warnings"].as_array().is_some_and(|w| {
            w.iter()
                .any(|w| w.as_str().is_some_and(|w| w.starts_with("Metadata")))
        });
        if warned != want_warning {
            panic!("{label}: warnings {}", json["warnings"]);
        }

        let requests = server.received_requests().await.unwrap();
        let Some(upload) = requests
            .iter()
            .find(|r| r.method.as_str() == "POST" && r.url.path() == "/api/assets")
        else {
            panic!("{label}: nothing uploaded");
        };
        let body = String::from_utf8_lossy(&upload.body);
        let created = multipart_text(&body, "fileCreatedAt");
        if created != Some("2026-01-01T00:00:00Z") {
            panic!("{label}: fileCreatedAt {created:?}");
        }
        if let Some(capture) = capture
            && !body.contains(capture)
        {
            panic!("{label}: upload lacks DateTimeOriginal {capture}");
        }
    }
}

async fn mock_upload(server: &MockServer, new_id: Uuid, update_status: u16) {
    Mock::given(method("POST"))
        .and(path("/api/assets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": new_id,
            "status": "created"
        })))
        .mount(server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .respond_with(ResponseTemplate::new(update_status))
        .mount(server)
        .await;
}

async fn mock_extraction(server: &MockServer, new_id: Uuid, pending_polls: u64) {
    let asset = |width: Option<u32>| {
        ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({ "id": new_id, "width": width }))
    };
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{new_id}")))
        .respond_with(asset(None))
        .up_to_n_times(pending_polls)
        .with_priority(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{new_id}")))
        .respond_with(asset(Some(64)))
        .with_priority(2)
        .mount(server)
        .await;
}

#[tokio::test]
async fn export_immich_copies_immich_metadata_to_the_edit() {
    let new_id = Uuid::new_v4();
    let everything = serde_json::json!({
        "ids": [new_id],
        "dateTimeOriginal": "2026-01-01T00:00:00Z",
        "timeZone": "Europe/Oslo",
        "latitude": 59.91,
        "longitude": 10.75,
        "description": "Harbour at dawn",
        "rating": 4
    });
    let mut no_location = everything.clone();
    if let Some(fields) = no_location.as_object_mut() {
        fields.remove("latitude");
        fields.remove("longitude");
    }
    for (metadata, update_status, expected, want_warning) in [
        ("all", 204, Some(&everything), false),
        ("no-location", 204, Some(&no_location), false),
        ("none", 204, None, false),
        ("all", 400, Some(&everything), true),
    ] {
        let label = format!("{metadata} {update_status}");
        let server = MockServer::start().await;
        mock_original_with(
            &server,
            asset_id(),
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(plain_jpeg()),
        )
        .await;
        mock_asset_detail(&server).await;
        mock_upload(&server, new_id, update_status).await;
        mock_extraction(&server, new_id, 2).await;

        let resp = test_app(&server)
            .await
            .oneshot(json_request(
                "POST",
                &format!("/api/assets/{}/export/immich", asset_id()),
                serde_json::json!({ "edits": {}, "metadata": metadata }),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("{label}: status {}", resp.status());
        }
        let json = body_json(resp).await;
        let warned = json["warnings"].as_array().is_some_and(|w| {
            w.iter().any(|w| {
                w.as_str()
                    .is_some_and(|w| w.starts_with("Metadata copy failed"))
            })
        });
        if warned != want_warning {
            panic!("{label}: warnings {}", json["warnings"]);
        }

        let requests = server.received_requests().await.unwrap();
        let polls: Vec<usize> = requests
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.method.as_str() == "GET" && r.url.path() == format!("/api/assets/{new_id}")
            })
            .map(|(i, _)| i)
            .collect();
        let first_update = requests
            .iter()
            .position(|r| r.method.as_str() == "PUT" && r.url.path() == "/api/assets");
        let waited = match (polls.last(), first_update) {
            (Some(poll), Some(update)) => polls.len() == 3 && *poll < update,
            (None, None) => true,
            _ => false,
        };
        if !waited {
            panic!("{label}: polls {polls:?}, update at {first_update:?}");
        }
        let updates: Vec<serde_json::Value> = requests
            .iter()
            .filter(|r| r.method.as_str() == "PUT" && r.url.path() == "/api/assets")
            .map(|r| serde_json::from_slice(&r.body).unwrap())
            .collect();
        let expected: Vec<serde_json::Value> = expected.into_iter().cloned().collect();
        if updates != expected {
            panic!("{label}: updates {updates:?}");
        }
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
async fn stack_primary_update_uses_patch() {
    let server = MockServer::start().await;
    let stack = Uuid::new_v4();
    let primary = asset_id();
    Mock::given(method("PATCH"))
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
