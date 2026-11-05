use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn assert_exported(status: StatusCode, json: &Value, upload: Uuid, replaced: Option<Uuid>) {
    let ok = status == StatusCode::OK
        && json["asset_id"] == upload.to_string()
        && json["warnings"] == json!([])
        && json.get("replaced_asset_id") == replaced.map(|id| json!(id)).as_ref();
    if !ok {
        panic!("{status} {json}");
    }
}

#[tokio::test]
async fn keep_both_uploads_again_and_records_the_newest_export() {
    let conflicts = Conflicts::start(true).await;
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    conflicts.uploads_as(first, "created").await;
    conflicts.uploads_as(second, "created").await;

    let (status, json) = conflicts
        .export(Some("key-1"), &conflict_body("keep_both"))
        .await;
    assert_exported(status, &json, first, None);
    assert_eq!(conflicts.recorded("").await, Some(first));

    let (status, json) = conflicts.export(None, &json!({ "metadata": "none" })).await;
    assert_exported(status, &json, second, None);
    assert_eq!(conflicts.recorded("").await, Some(second));
    assert_eq!(conflicts.uploads().await, 2);
    assert_eq!(
        conflicts
            .calls("GET", &format!("/api/assets/{first}"))
            .await,
        0
    );
}

#[tokio::test]
async fn skip_returns_the_previous_export_without_rendering_or_uploading() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;

    let (status, json) = conflicts
        .export(Some("key-1"), &conflict_body("skip"))
        .await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(
        json,
        json!({
            "asset_id": previous,
            "filename": EDIT_NAME,
            "status": "skipped",
            "warnings": [],
            "original_trashed": false,
        })
    );
    assert_eq!(conflicts.uploads().await, 0);
    let original = format!("/api/assets/{}/original", asset_id());
    assert_eq!(conflicts.calls("GET", &original).await, 0);
    let row = conflicts
        .state
        .edits
        .get_export_job(export_job(master(), "key-1"))
        .await
        .unwrap();
    assert!(row.is_none());
    assert_eq!(conflicts.recorded("").await, Some(previous));
}

#[tokio::test]
async fn skip_still_refuses_an_invalid_filename_template() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Live).await;

    let (status, json) = conflicts
        .export(
            Some("key-1"),
            &json!({ "on_conflict": "skip", "filename_template": "{name" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{json}");
}

#[tokio::test]
async fn replace_trashes_the_previous_export_after_uploading() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    conflicts.expect_replace(previous, upload, 1).await;
    let body = conflict_body("replace");

    let (status, json) = conflicts.export(Some("key-1"), &body).await;
    let replay = conflicts.export(Some("key-1"), &body).await;

    assert_exported(status, &json, upload, Some(previous));
    assert_eq!(replay, (status, json));
    assert_eq!(conflicts.recorded("").await, Some(upload));
    let row = conflicts
        .state
        .edits
        .get_export_job(export_job(master(), "key-1"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.replaced_asset_id, Some(previous));
    assert_eq!(row.previous_asset_id, Some(previous));
}

#[tokio::test]
async fn replace_leaves_an_upload_immich_matched_to_the_previous_export() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    conflicts.uploads_as(previous, "duplicate").await;

    let (status, json) = conflicts
        .export(Some("key-1"), &conflict_body("replace"))
        .await;

    assert_exported(status, &json, previous, None);
    assert_eq!(json["status"], "duplicate");
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
    assert!(conflicts.trashed().await.is_empty());
    assert_eq!(conflicts.recorded("").await, Some(previous));
}

#[tokio::test]
async fn replace_skips_an_upload_immich_matched_to_the_source_original() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    conflicts.uploads_as(asset_id(), "duplicate").await;

    let (status, json) = conflicts
        .export(Some("key-1"), &conflict_body("replace"))
        .await;

    assert_exported(status, &json, asset_id(), None);
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
    assert!(conflicts.trashed().await.is_empty());
    assert_eq!(conflicts.recorded("").await, Some(previous));
}

#[tokio::test]
async fn a_previous_export_gone_from_immich_is_no_conflict() {
    for gone in [404, 400] {
        let conflicts = Conflicts::start(true).await;
        conflicts.previous("", PreviousExport::Answers(gone)).await;
        let upload = Uuid::new_v4();
        conflicts.uploads_as(upload, "created").await;

        let (status, json) = conflicts
            .export(Some("key-1"), &conflict_body("skip"))
            .await;

        assert_exported(status, &json, upload, None);
        assert_eq!(conflicts.uploads().await, 1, "{gone}");
        assert_eq!(conflicts.recorded("").await, Some(upload), "{gone}");
    }
}

#[tokio::test]
async fn a_trashed_previous_export_is_no_conflict() {
    for policy in ["skip", "replace"] {
        let conflicts = Conflicts::start(true).await;
        conflicts.previous("", PreviousExport::Trashed).await;
        let upload = Uuid::new_v4();
        conflicts.uploads_as(upload, "created").await;

        let (status, json) = conflicts
            .export(Some("key-1"), &conflict_body(policy))
            .await;

        assert_exported(status, &json, upload, None);
        assert!(conflicts.trashed().await.is_empty(), "{policy}");
        assert_eq!(conflicts.recorded("").await, Some(upload), "{policy}");
    }
}

#[tokio::test]
async fn replace_is_refused_up_front_without_the_immich_trash() {
    let conflicts = Conflicts::start(false).await;
    conflicts.previous("", PreviousExport::Live).await;

    let (status, json) = conflicts
        .export(Some("key-1"), &conflict_body("replace"))
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{json}");
    assert_eq!(json["code"], "trash_disabled");

    let (status, _) = conflicts
        .export(Some("key-2"), &conflict_body("overwrite"))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(conflicts.uploads().await, 0);
}

#[tokio::test]
async fn a_copy_is_tracked_apart_from_its_master() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let copy = conflicts
        .state
        .edits
        .create_copy(test_user_id(), asset_id(), None)
        .await
        .unwrap()
        .id;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    let skip = conflict_body("skip");

    let (status, json) = conflicts.export_asset(copy, Some("key-1"), &skip).await;
    assert_exported(status, &json, upload, None);
    let (status, json) = conflicts.export_asset(copy, Some("key-2"), &skip).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["status"], "skipped");
    assert_eq!(json["asset_id"], upload.to_string());

    assert_eq!(conflicts.uploads().await, 1);
    assert_eq!(conflicts.recorded_for(copy, "").await, Some(upload));
    assert_eq!(conflicts.recorded("").await, Some(previous));
}
