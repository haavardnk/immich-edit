use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::services::edits_store::EditWrite;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn replace_with(extra: Value) -> Value {
    let mut body = conflict_body("replace");
    body.as_object_mut()
        .unwrap()
        .extend(extra.as_object().cloned().unwrap_or_default());
    body
}

async fn kept(conflicts: &Conflicts, upload: Uuid, request: &Value) -> Value {
    let (status, json) = conflicts.export(Some("key-1"), request).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["asset_id"], upload.to_string());
    assert!(json.get("replaced_asset_id").is_none(), "{json}");
    assert_eq!(conflicts.recorded("").await, Some(upload));
    json
}

#[tokio::test]
async fn conflict_replace_runs_before_replace_original() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    conflicts.expect_replace(previous, upload, 1).await;
    conflicts.expect_replace(asset_id(), upload, 1).await;

    let (status, json) = conflicts
        .export(
            Some("key-1"),
            &replace_with(json!({ "replace_original": true })),
        )
        .await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["replaced_asset_id"], previous.to_string());
    assert_eq!(json["original_trashed"], true);
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(
        conflicts.trashed().await,
        [json!([previous]), json!([asset_id()])]
    );
    assert_eq!(conflicts.recorded("").await, Some(upload));
}

#[tokio::test]
async fn a_failed_previous_export_replace_does_not_block_replace_original() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    Mock::given(method("PUT"))
        .and(path("/api/assets/copy"))
        .and(body_partial_json(json!({ "sourceId": previous })))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "message": "Not allowed" })))
        .expect(1)
        .mount(&conflicts.server)
        .await;
    conflicts.expect_replace(asset_id(), upload, 1).await;

    let json = kept(
        &conflicts,
        upload,
        &replace_with(json!({ "replace_original": true })),
    )
    .await;

    assert_eq!(json["original_trashed"], true);
    let warnings = warnings(&json);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("Previous export kept: copying its albums and links failed"),
        "{warnings:?}"
    );
    assert_eq!(conflicts.trashed().await, [json!([asset_id()])]);
}

#[tokio::test]
async fn a_failed_post_upload_step_keeps_the_previous_export() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    let album = Uuid::new_v4();
    Mock::given(method("PUT"))
        .and(path(format!("/api/albums/{album}/assets")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": upload, "success": false, "error": "no_permission" }
        ])))
        .mount(&conflicts.server)
        .await;

    let json = kept(
        &conflicts,
        upload,
        &replace_with(json!({ "album_ids": [album] })),
    )
    .await;

    assert_eq!(
        warnings(&json),
        [
            format!("Album {album} failed: no_permission").as_str(),
            "Previous export kept: an earlier step failed",
        ]
    );
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
    assert!(conflicts.trashed().await.is_empty());
}

#[tokio::test]
async fn a_previous_export_of_another_user_is_kept() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Foreign).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;

    let json = kept(&conflicts, upload, &conflict_body("replace")).await;

    assert_eq!(
        warnings(&json),
        ["Previous export kept: it belongs to another Immich user"]
    );
    assert!(conflicts.trashed().await.is_empty());
}

#[tokio::test]
async fn a_previous_export_with_its_own_edits_is_kept() {
    for virtual_copy in [false, true] {
        let conflicts = Conflicts::start(true).await;
        let previous = conflicts.previous("", PreviousExport::Live).await;
        let edits = &conflicts.state.edits;
        if virtual_copy {
            edits
                .create_copy(test_user_id(), previous, None)
                .await
                .unwrap();
        } else {
            edits
                .put(
                    test_user_id(),
                    AssetKey::master(previous),
                    EditWrite::default(),
                )
                .await
                .unwrap();
        }
        let upload = Uuid::new_v4();
        conflicts.uploads_as(upload, "created").await;

        let json = kept(&conflicts, upload, &conflict_body("replace")).await;

        assert_eq!(
            warnings(&json),
            ["Previous export kept: it has its own edits"],
            "{virtual_copy}"
        );
        assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
        assert!(conflicts.trashed().await.is_empty());
    }
}
