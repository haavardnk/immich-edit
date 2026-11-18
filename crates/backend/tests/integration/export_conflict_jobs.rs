use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::services::edits_store::ExportJobKey;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

async fn create_job(conflicts: &Conflicts, kind: &str, params: Value) -> StatusCode {
    conflicts
        .app
        .clone()
        .oneshot(create_job_request(kind, params, &[asset_id().to_string()]))
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn conflicts_are_kept_apart_per_preset() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("web", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    let skip = conflict_body("skip");
    let params = json!({ "variants": [
        preset_variant("Web", skip.clone()),
        preset_variant("Print", skip),
    ] });
    assert_eq!(
        create_job(&conflicts, "export_immich", params).await,
        StatusCode::OK
    );

    let results = run_all(&conflicts.state).await;

    let outcomes: Vec<(&str, &str, &str)> = results
        .iter()
        .map(|r| {
            (
                r["preset"].as_str().unwrap_or_default(),
                r["status"].as_str().unwrap_or_default(),
                r["asset_id"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        outcomes,
        [
            ("Web", "skipped", previous.to_string().as_str()),
            ("Print", "created", upload.to_string().as_str()),
        ]
    );
    assert_eq!(conflicts.uploads().await, 1);
    assert_eq!(conflicts.recorded("web").await, Some(previous));
    assert_eq!(conflicts.recorded("print").await, Some(upload));
    assert_eq!(conflicts.recorded("").await, None);
}

#[tokio::test]
async fn a_previous_export_shared_by_another_preset_is_kept() {
    let conflicts = Conflicts::start(true).await;
    let shared = Uuid::new_v4();
    conflicts.track(master(), "web", shared).await;
    conflicts.track(master(), "print", shared).await;
    conflicts.answer(shared, PreviousExport::Live).await;
    let (web, print) = (Uuid::new_v4(), Uuid::new_v4());
    conflicts.uploads_as(web, "created").await;
    conflicts.uploads_as(print, "created").await;
    conflicts.accept_replaces().await;
    let replace = conflict_body("replace");
    let params = json!({ "variants": [
        preset_variant("Web", replace.clone()),
        preset_variant("Print", replace),
    ] });
    assert_eq!(
        create_job(&conflicts, "export_immich", params).await,
        StatusCode::OK
    );

    let results = run_all(&conflicts.state).await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["preset"], "Web");
    assert_eq!(results[0]["asset_id"], web.to_string());
    assert!(results[0].get("replaced_asset_id").is_none());
    assert_eq!(
        warnings(&results[0]),
        ["Previous export kept: another export still uses it"]
    );
    assert_eq!(results[1]["preset"], "Print");
    assert_eq!(results[1]["asset_id"], print.to_string());
    assert_eq!(results[1]["replaced_asset_id"], shared.to_string());
    assert_eq!(warnings(&results[1]), Vec::<&str>::new());
    assert_eq!(conflicts.calls("PUT", "/api/assets/copy").await, 1);
    assert_eq!(conflicts.trashed().await, [json!([shared])]);
    assert_eq!(conflicts.recorded("web").await, Some(web));
    assert_eq!(conflicts.recorded("print").await, Some(print));
}

#[tokio::test]
async fn a_replace_job_item_reports_and_keeps_the_replaced_export() {
    let conflicts = Conflicts::start(true).await;
    let previous = conflicts.previous("", PreviousExport::Live).await;
    let upload = Uuid::new_v4();
    conflicts.uploads_as(upload, "created").await;
    conflicts.expect_replace(previous, upload, 1).await;
    let job = conflicts
        .app
        .clone()
        .oneshot(create_job_request(
            "export_immich",
            conflict_body("replace"),
            &[asset_id().to_string()],
        ))
        .await
        .unwrap();
    let job_id = created_job_id(job).await;

    let results = run_all(&conflicts.state).await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["asset_id"], upload.to_string());
    assert_eq!(results[0]["replaced_asset_id"], previous.to_string());
    assert_eq!(conflicts.recorded("").await, Some(upload));
    let key = format!("job-{job_id}");
    let row = conflicts
        .state
        .edits
        .get_export_job(ExportJobKey {
            owner: test_user_id(),
            asset_id: AssetKey::master(asset_id()),
            key: &key,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.replaced_asset_id, Some(previous));
}

#[tokio::test]
async fn replace_jobs_need_the_immich_trash() {
    let conflicts = Conflicts::start(false).await;
    let replace = conflict_body("replace");

    for (kind, params, want) in [
        ("export_immich", replace.clone(), StatusCode::CONFLICT),
        (
            "export_immich",
            json!({ "variants": [preset_variant("Web", replace.clone())] }),
            StatusCode::CONFLICT,
        ),
        (
            "export_immich",
            conflict_body("overwrite"),
            StatusCode::BAD_REQUEST,
        ),
        ("export_immich", conflict_body("skip"), StatusCode::OK),
        ("download_zip", replace.clone(), StatusCode::OK),
    ] {
        assert_eq!(
            create_job(&conflicts, kind, params.clone()).await,
            want,
            "{kind} {params}"
        );
    }
}

#[tokio::test]
async fn a_rebind_forgets_export_outputs() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Live).await;
    conflicts.previous("web", PreviousExport::Live).await;

    rebind_instance(&conflicts.server, &conflicts.state).await;

    assert_eq!(conflicts.recorded("").await, None);
    assert_eq!(conflicts.recorded("web").await, None);
}
