use crate::common::*;
use immich_edit_backend::services::edits_store::ExportJobBaseline;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

const STALE_HASH: &str = "hash-of-older-edits";

fn no_baseline() -> ExportJobBaseline {
    ExportJobBaseline {
        previous_asset_id: None,
        replace_asset_id: None,
    }
}

async fn queued_export(conflicts: &Conflicts) -> String {
    let resp = conflicts
        .app
        .clone()
        .oneshot(create_job_request(
            "export_immich",
            conflict_body("keep_both"),
            &[asset_id().to_string()],
        ))
        .await
        .unwrap();
    format!("job-{}", created_job_id(resp).await)
}

#[tokio::test]
async fn a_job_item_takes_over_its_own_stale_reservation() {
    let conflicts = Conflicts::start(true).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    let key = queued_export(&conflicts).await;
    let reserved = conflicts
        .state
        .edits
        .reserve_export_job(export_job(master(), &key), STALE_HASH, no_baseline())
        .await
        .unwrap();
    assert!(reserved);

    let results = run_all(&conflicts.state).await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["asset_id"], upload.to_string());
    assert_eq!(conflicts.uploads().await, 1);
}

#[tokio::test]
async fn a_job_item_finishes_its_earlier_upload_after_the_edits_changed() {
    let conflicts = Conflicts::start(true).await;
    let earlier = Uuid::new_v4();
    conflicts.answer(earlier, PreviousExport::Live).await;
    let key = queued_export(&conflicts).await;
    let job = export_job(master(), &key);
    let edits = &conflicts.state.edits;
    assert!(
        edits
            .reserve_export_job(job, STALE_HASH, no_baseline())
            .await
            .unwrap()
    );
    edits
        .put_export_job_uploaded(job, STALE_HASH, earlier, EDIT_NAME, "created", &[])
        .await
        .unwrap();

    let results = run_all(&conflicts.state).await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["asset_id"], earlier.to_string());
    assert_eq!(results[0]["status"], json!("created"));
    assert_eq!(conflicts.uploads().await, 0);
}
