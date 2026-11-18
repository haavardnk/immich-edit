use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::services::edits_store::ExportJobBaseline;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn resumed(
    conflicts: &Conflicts,
    policy: &str,
    upload: Uuid,
    baseline: ExportJobBaseline,
) -> Value {
    let body = conflict_body(policy);
    conflicts.answer(upload, PreviousExport::Live).await;
    conflicts
        .seed_uploaded("key-1", &body, upload, baseline)
        .await;

    let (status, json) = conflicts.export(Some("key-1"), &body).await;
    let replay = conflicts.export(Some("key-1"), &body).await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(replay, (status, json.clone()));
    assert_eq!(json["asset_id"], upload.to_string());
    assert_eq!(json["status"], "created");
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(conflicts.uploads().await, 0);
    assert_eq!(conflicts.recorded("").await, Some(upload));
    json
}

async fn expect_copy(conflicts: &Conflicts, source: Uuid, target: Uuid) {
    Mock::given(method("PUT"))
        .and(path("/api/assets/copy"))
        .and(body_partial_json(
            json!({ "sourceId": source, "targetId": target }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&conflicts.server)
        .await;
}

async fn copy_sources(conflicts: &Conflicts) -> Vec<Value> {
    conflicts
        .server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == "PUT" && r.url.path() == "/api/assets/copy")
        .map(|r| serde_json::from_slice::<Value>(&r.body).unwrap()["sourceId"].clone())
        .collect()
}

#[tokio::test]
async fn a_resumed_replace_trashes_the_previous_export_once() {
    for answer in [PreviousExport::Live, PreviousExport::Trashed] {
        let conflicts = Conflicts::start(true).await;
        let previous = conflicts.previous("", answer).await;
        let upload = Uuid::new_v4();
        conflicts.expect_replace(previous, upload, 1).await;

        let json = resumed(&conflicts, "replace", upload, baseline(previous, true)).await;

        assert_eq!(json["replaced_asset_id"], previous.to_string());
        assert_eq!(conflicts.trashed().await, [json!([previous])]);
    }
}

#[tokio::test]
async fn a_previous_export_trashed_before_the_upload_is_not_replaced_on_resume() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Trashed).await;
    let upload = Uuid::new_v4();

    let json = resumed(&conflicts, "replace", upload, baseline(previous, false)).await;

    assert!(json.get("replaced_asset_id").is_none(), "{json}");
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
    assert!(conflicts.trashed().await.is_empty());
}

#[tokio::test]
async fn a_resume_leaves_an_export_another_export_already_replaced() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    let body = conflict_body("replace");
    conflicts
        .seed_uploaded("job-1", &body, first, baseline(previous, true))
        .await;
    conflicts.answer(first, PreviousExport::Live).await;
    conflicts.uploads_as(second, "created").await;
    expect_copy(&conflicts, previous, second).await;
    conflicts.accept_replaces().await;

    let (status, editor) = conflicts.export(Some("editor"), &body).await;
    assert_eq!(status, StatusCode::OK, "{editor}");
    assert_eq!(editor["asset_id"], second.to_string());
    assert_eq!(editor["replaced_asset_id"], previous.to_string());
    assert_eq!(conflicts.recorded("").await, Some(second));

    let (status, job) = conflicts.export(Some("job-1"), &body).await;

    assert_eq!(status, StatusCode::OK, "{job}");
    assert_eq!(job["asset_id"], first.to_string());
    assert!(job.get("replaced_asset_id").is_none(), "{job}");
    assert_eq!(job["warnings"], json!([]));
    assert_eq!(conflicts.trashed().await, [json!([previous])]);
    assert_eq!(copy_sources(&conflicts).await, [json!(previous)]);
    assert_eq!(conflicts.uploads().await, 1);
    assert_eq!(conflicts.recorded("").await, Some(second));
}

#[tokio::test]
async fn skip_on_resume_finishes_and_records_the_upload() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();

    let json = resumed(&conflicts, "skip", upload, baseline(previous, false)).await;

    assert!(json.get("replaced_asset_id").is_none(), "{json}");
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
    assert!(conflicts.trashed().await.is_empty());
}

#[tokio::test]
async fn a_previous_export_gone_on_resume_is_left_alone() {
    for gone in [404, 400] {
        let conflicts = Conflicts::start(true).await;
        let previous = conflicts.previous("", PreviousExport::Answers(gone)).await;
        let upload = Uuid::new_v4();

        let json = resumed(&conflicts, "replace", upload, baseline(previous, true)).await;

        assert!(json.get("replaced_asset_id").is_none(), "{gone} {json}");
        assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 0);
        assert!(conflicts.trashed().await.is_empty(), "{gone}");
    }
}

#[tokio::test]
async fn concurrent_replaces_leave_one_live_tracked_export() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let uploads = [Uuid::new_v4(), Uuid::new_v4()];
    for upload in uploads {
        conflicts.uploads_as(upload, "created").await;
    }
    conflicts.accept_replaces().await;
    let body = conflict_body("replace");

    let (a, b) = tokio::join!(
        conflicts.export(Some("a"), &body),
        conflicts.export(Some("b"), &body)
    );

    for (status, json) in [&a, &b] {
        assert_eq!(*status, StatusCode::OK, "{json}");
        assert_eq!(json["warnings"], json!([]), "{json}");
    }
    let trashed = conflicts.trashed().await;
    assert_eq!(trashed.len(), 2, "{trashed:?}");
    assert_eq!(trashed[0], json!([previous]));
    let superseded: Uuid = serde_json::from_value(trashed[1][0].clone()).unwrap();
    let live = conflicts.recorded("").await.unwrap();
    assert_ne!(superseded, live);
    let mut ends = [superseded, live];
    ends.sort();
    let mut want = uploads;
    want.sort();
    assert_eq!(ends, want);
    let mut replaced: Vec<Value> = [&a, &b]
        .iter()
        .map(|(_, json)| json["replaced_asset_id"].clone())
        .collect();
    replaced.sort_by_key(|id| *id != json!(previous));
    assert_eq!(replaced, [json!(previous), json!(superseded)]);
}

#[tokio::test]
async fn concurrent_keep_both_and_replace_track_the_newest_live_export() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let uploads = [Uuid::new_v4(), Uuid::new_v4()];
    for upload in uploads {
        conflicts.uploads_as(upload, "created").await;
    }
    conflicts.accept_replaces().await;
    let (keep_both, replace_body) = (conflict_body("keep_both"), conflict_body("replace"));

    let (keep, replace) = tokio::join!(
        conflicts.export(Some("keep"), &keep_both),
        conflicts.export(Some("replace"), &replace_body)
    );

    for (status, json) in [&keep, &replace] {
        assert_eq!(*status, StatusCode::OK, "{json}");
        assert_eq!(json["warnings"], json!([]), "{json}");
    }
    assert!(keep.1.get("replaced_asset_id").is_none(), "{}", keep.1);
    let replaced = replace.1["replaced_asset_id"].clone();
    assert!(
        replaced == json!(previous) || replaced == json!(uploads[0]),
        "{replaced}"
    );
    assert_eq!(conflicts.trashed().await, [json!([replaced])]);
    assert_eq!(conflicts.recorded("").await, Some(uploads[1]));
}
