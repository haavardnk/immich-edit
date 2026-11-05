use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::services::job_store::{JobItemStatus, JobStatus};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

fn web_and_print() -> Value {
    json!({ "variants": [
        preset_variant("Web", json!({ "filename_template": "{name}_{preset}", "metadata": "none" })),
        preset_variant("Print", json!({ "filename_template": "{name}_{preset}", "metadata": "none", "format": "png" })),
    ] })
}

async fn zip_names(app: axum::Router, job_id: Uuid) -> Vec<String> {
    let resp = app.oneshot(download_request(job_id)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body_bytes(resp).await)).unwrap();
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn a_zip_job_exports_every_photo_with_every_preset() {
    let server = MockServer::start().await;
    let assets = [Uuid::new_v4(), Uuid::new_v4()];
    mock_named_asset(&server, assets[0], "DSC0001.ARW").await;
    mock_named_asset(&server, assets[1], "DSC0002.ARW").await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token);
    let ids: Vec<String> = assets.iter().map(Uuid::to_string).collect();

    let job_id = created_job_id(
        app.clone()
            .oneshot(create_job_request("download_zip", web_and_print(), &ids))
            .await
            .unwrap(),
    )
    .await;

    let items = state.jobs.list_items(job_id).await.unwrap();
    let order: Vec<(String, i64, i64)> = items
        .iter()
        .map(|item| (item.asset_id.clone(), item.position, item.variant))
        .collect();
    assert_eq!(
        order,
        [
            (ids[0].clone(), 1, 0),
            (ids[0].clone(), 1, 1),
            (ids[1].clone(), 2, 0),
            (ids[1].clone(), 2, 1),
        ]
    );
    let results = run_all(&state).await;
    let presets: Vec<&str> = results
        .iter()
        .filter_map(|result| result["preset"].as_str())
        .collect();
    assert_eq!(presets, ["Web", "Print", "Web", "Print"]);
    assert_eq!(
        state.jobs.get_job(job_id).await.unwrap().unwrap().status,
        JobStatus::Completed
    );
    let detail = body_json(
        app.clone()
            .oneshot(get(&format!("/api/jobs/{job_id}")))
            .await
            .unwrap(),
    )
    .await;
    let variants: Vec<i64> = detail["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["variant"].as_i64())
        .collect();
    assert_eq!(variants, [0, 1, 0, 1]);

    assert_eq!(
        zip_names(app, job_id).await,
        [
            "DSC0001_Print.png",
            "DSC0001_Web.jpg",
            "DSC0002_Print.png",
            "DSC0002_Web.jpg"
        ]
    );
}

#[tokio::test]
async fn presets_without_a_template_name_their_files_by_preset() {
    let server = MockServer::start().await;
    mock_named_asset(&server, asset_id(), "DSC0001.ARW").await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token);
    let params = json!({ "variants": [
        preset_variant("Small", json!({ "metadata": "none" })),
        preset_variant("Large", json!({ "metadata": "none", "filename_template": "" })),
    ] });

    let job_id = created_job_id(
        app.clone()
            .oneshot(create_job_request(
                "download_zip",
                params,
                &[asset_id().to_string()],
            ))
            .await
            .unwrap(),
    )
    .await;
    run_all(&state).await;

    assert_eq!(
        zip_names(app, job_id).await,
        ["DSC0001_Large.jpg", "DSC0001_Small.jpg"]
    );
}

#[tokio::test]
async fn a_zip_job_skips_the_immich_replace_checks() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, false).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state), token);
    let replace = json!({ "replace_original": true, "stack_with_original": true });
    let params = json!({ "variants": [
        preset_variant("Web", replace.clone()),
        preset_variant("Print", replace),
    ] });

    let resp = app
        .oneshot(create_job_request(
            "download_zip",
            params,
            &[asset_id().to_string()],
        ))
        .await
        .unwrap();

    created_job_id(resp).await;
    let features = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/api/server/features")
        .count();
    assert_eq!(features, 0);
}

#[tokio::test]
async fn variant_jobs_reject_malformed_variants() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let web = || preset_variant("Web", json!({}));
    let many = |n: usize| -> Vec<Value> {
        (0..n)
            .map(|i| preset_variant(&format!("P{i}"), json!({})))
            .collect()
    };
    for (kind, params, message) in [
        (
            "download_zip",
            json!({ "variants": [web()], "format": "png" }),
            "unexpected key beside variants: format",
        ),
        (
            "download_zip",
            json!({ "variants": many(0) }),
            "a job exports 1 to 16 presets",
        ),
        (
            "export_immich",
            json!({ "variants": many(17) }),
            "a job exports 1 to 16 presets",
        ),
        (
            "download_zip",
            json!({ "variants": [preset_variant(" ", json!({}))] }),
            "preset name required",
        ),
        (
            "export_immich",
            json!({ "variants": [web(), preset_variant("WEB", json!({}))] }),
            "preset WEB is listed twice",
        ),
        (
            "download_zip",
            json!({ "variants": [
                preset_variant("A/B", json!({})),
                preset_variant("A:B", json!({})),
            ] }),
            "presets A/B and A:B give the same file name",
        ),
        (
            "export_immich",
            json!({ "variants": [
                preset_variant("Replace", json!({ "replace_original": true })),
                web(),
            ] }),
            "replace original exports one version of each photo",
        ),
        (
            "download_zip",
            json!({ "variants": [preset_variant("Web", json!({ "format": "bmp" }))] }),
            "Web: invalid export params",
        ),
        (
            "export_immich",
            json!({ "variants": [preset_variant("Web", json!({
                "replace_original": true,
                "stack_with_original": true
            }))] }),
            "replace original and stack with original cannot be combined",
        ),
    ] {
        let resp = app
            .clone()
            .oneshot(create_job_request(
                kind,
                params.clone(),
                &[asset_id().to_string()],
            ))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{params}");
        let got = body_json(resp).await["message"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(got.starts_with(message), "{params}: {got}");
    }
}

#[tokio::test]
async fn presets_count_against_the_item_limit() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let ids: Vec<String> = (0..5_001).map(|_| Uuid::new_v4().to_string()).collect();

    let resp = app
        .oneshot(create_job_request("download_zip", web_and_print(), &ids))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(resp).await["message"], "too many items");
}

#[tokio::test]
async fn retrying_a_failed_preset_keeps_its_variant() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token);
    let job_id = created_job_id(
        app.clone()
            .oneshot(create_job_request(
                "download_zip",
                web_and_print(),
                &[asset_id().to_string()],
            ))
            .await
            .unwrap(),
    )
    .await;
    complete_next(&state).await;
    let failed = fail_next(&state).await;
    assert_eq!(failed.variant, 1);

    let resp = app.oneshot(retry_request(job_id)).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let items = state.jobs.list_items(job_id).await.unwrap();
    let states: Vec<(i64, JobItemStatus)> = items
        .iter()
        .map(|item| (item.variant, item.status))
        .collect();
    assert_eq!(
        states,
        [(0, JobItemStatus::Completed), (1, JobItemStatus::Pending)]
    );
    let reclaimed = state.jobs.claim_next_item().await.unwrap().unwrap();
    assert_eq!((reclaimed.id, reclaimed.variant), (failed.id, 1));
}
