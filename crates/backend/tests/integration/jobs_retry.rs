use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::error::AppError;
use immich_edit_backend::services::export::{
    DOWNLOAD_ZIP_KIND, EXPORT_JOB_KIND, build_zip_archive, confirm_zip_archive, zip_archive_path,
    zip_job_dir,
};
use immich_edit_backend::services::job_store::{JobItemStatus, JobRecord, JobStatus};
use immich_edit_backend::state::AppState;
use serde_json::{Value, json};
use std::path::PathBuf;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

struct ZipJob {
    job: JobRecord,
    epoch: i64,
    dir: PathBuf,
    archive: PathBuf,
}

async fn finished_zip_job(state: &AppState, token: &str) -> ZipJob {
    let ctx = state.auth.authenticate(token).await.unwrap().unwrap();
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .assets(&[asset_id(), Uuid::new_v4()])
        .create(state, token)
        .await;
    let dir = zip_job_dir(state, ctx.server_epoch, ctx.user.id, job.id).unwrap();
    let archive = zip_archive_path(state, ctx.server_epoch, ctx.user.id, job.id).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("first.jpg"), b"first").unwrap();
    complete_next(state).await;
    fail_next(state).await;
    ZipJob {
        job: state.jobs.get_job(job.id).await.unwrap().unwrap(),
        epoch: ctx.server_epoch,
        dir,
        archive,
    }
}

async fn expect_retry_conflict(app: axum::Router, id: Uuid, message: &str) {
    let res = app.oneshot(retry_request(id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT, "{message}");
    assert_eq!(body_json(res).await["message"], message);
}

async fn zip_entries(res: axum::response::Response) -> usize {
    let bytes = body_bytes(res).await;
    zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .unwrap()
        .len()
}

async fn job_status(state: &AppState, id: Uuid) -> JobStatus {
    state.jobs.get_job(id).await.unwrap().unwrap().status
}

#[tokio::test]
async fn retrying_a_finished_job_requeues_only_its_failed_items() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let second = Uuid::new_v4();
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .assets(&[asset_id(), second])
        .create(&state, &token)
        .await;
    let completed = complete_next(&state).await;
    let failed = fail_next(&state).await;
    let app = wrap_auth(router(state.clone()), token);

    let res = app.clone().oneshot(retry_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body["id"], job.id.to_string());
    assert_eq!(body["status"], "pending");
    assert_eq!(body["completed"], 1);
    assert_eq!(body["failed"], 0);
    let detail = body_json(
        app.oneshot(get(&format!("/api/jobs/{}", job.id)))
            .await
            .unwrap(),
    )
    .await;
    let items = detail["items"].as_array().unwrap();
    let item = |id: Uuid| {
        items
            .iter()
            .find(|item| item["id"] == id.to_string())
            .unwrap()
    };
    assert_eq!(item(completed.id)["status"], "completed");
    assert_eq!(item(failed.id)["status"], "pending");
    assert_eq!(item(failed.id)["error"], Value::Null);
    assert!(state.jobs.job_credential(job.id).await.unwrap().is_some());
    let reclaimed = state.jobs.claim_next_item().await.unwrap().unwrap();
    assert_eq!(reclaimed.id, failed.id);
    assert_eq!(reclaimed.asset_id, second.to_string());
}

#[tokio::test]
async fn an_unfinished_job_cannot_be_retried() {
    for claimed in [false, true] {
        let server = MockServer::start().await;
        let state = test_state(&server).await;
        let token = seed_session(&server, &state).await;
        let job = JobSeed::new(DOWNLOAD_ZIP_KIND).create(&state, &token).await;
        if claimed {
            state.jobs.claim_next_item().await.unwrap().unwrap();
        }

        expect_retry_conflict(
            wrap_auth(router(state), token),
            job.id,
            "job is still running",
        )
        .await;
    }
}

#[tokio::test]
async fn a_second_retry_finds_the_job_running() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND).create(&state, &token).await;
    fail_next(&state).await;
    let app = wrap_auth(router(state.clone()), token);

    let first = app.clone().oneshot(retry_request(job.id)).await.unwrap();

    assert_eq!(first.status(), StatusCode::OK);
    expect_retry_conflict(app, job.id, "job is still running").await;
    let items = state.jobs.list_items(job.id).await.unwrap();
    assert_eq!(items[0].status, JobItemStatus::Pending);
}

#[tokio::test]
async fn a_cancelled_job_cannot_be_retried() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .assets(&[asset_id(), Uuid::new_v4()])
        .create(&state, &token)
        .await;
    fail_next(&state).await;
    assert!(state.jobs.cancel_job(job.id).await.unwrap());

    expect_retry_conflict(wrap_auth(router(state), token), job.id, "job was cancelled").await;
}

#[tokio::test]
async fn a_job_without_failures_cannot_be_retried() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND).create(&state, &token).await;
    complete_next(&state).await;

    expect_retry_conflict(
        wrap_auth(router(state), token),
        job.id,
        "job has no failed items",
    )
    .await;
}

#[tokio::test]
async fn a_job_from_a_previous_server_cannot_be_retried() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let ctx = state.auth.authenticate(&token).await.unwrap().unwrap();
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .server_epoch(ctx.server_epoch - 1)
        .create(&state, &token)
        .await;
    fail_next(&state).await;

    expect_retry_conflict(
        wrap_auth(router(state), token),
        job.id,
        "job belongs to a previous server",
    )
    .await;
}

#[tokio::test]
async fn another_members_job_cannot_be_retried() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let admin_token = seed_session(&server, &state).await;
    let member_token = seed_member_session(&server, &state).await;
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .create(&state, &member_token)
        .await;
    fail_next(&state).await;
    let app = wrap_auth(router(state.clone()), admin_token);

    let res = app.oneshot(retry_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let job = state.jobs.get_job(job.id).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.failed, 1);
}

#[tokio::test]
async fn an_export_retry_revalidates_its_params() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = JobSeed::new(EXPORT_JOB_KIND)
        .params(json!({ "replace_original": true, "stack_with_original": true }))
        .create(&state, &token)
        .await;
    fail_next(&state).await;
    let app = wrap_auth(router(state.clone()), token);

    let res = app.oneshot(retry_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(res).await["message"],
        "replace original and stack with original cannot be combined"
    );
    assert_eq!(job_status(&state, job.id).await, JobStatus::Failed);
    assert_eq!(
        state.jobs.list_items(job.id).await.unwrap()[0].status,
        JobItemStatus::Failed
    );
}

#[tokio::test]
async fn an_export_retry_validates_only_the_failed_items() {
    let server = MockServer::start().await;
    mock_trash_feature(&server, true).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let params = json!({ "replace_original": true });
    let both_versions = [asset_id().to_string(), format!("{}_1", asset_id())];
    let app = wrap_auth(router(state.clone()), token.clone());
    let rejected = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/jobs",
            json!({ "kind": EXPORT_JOB_KIND, "asset_ids": both_versions, "params": params }),
        ))
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(rejected).await["message"],
        "replace original takes one version of each photo"
    );
    let job = JobSeed::new(EXPORT_JOB_KIND)
        .params(params)
        .assets(&both_versions)
        .create(&state, &token)
        .await;
    complete_next(&state).await;
    fail_next(&state).await;

    let res = app.oneshot(retry_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(job_status(&state, job.id).await, JobStatus::Pending);
}

#[tokio::test]
async fn a_retried_zip_job_drops_only_its_cached_archive() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let zip = finished_zip_job(&state, &token).await;
    let app = wrap_auth(router(state.clone()), token);

    let first = app
        .clone()
        .oneshot(download_request(zip.job.id))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(zip_entries(first).await, 1);
    assert!(zip.archive.exists());

    let res = app
        .clone()
        .oneshot(retry_request(zip.job.id))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(!zip.archive.exists());
    assert!(zip.dir.join("first.jpg").exists());
    std::fs::write(zip.dir.join("second.jpg"), b"second").unwrap();
    complete_next(&state).await;
    let second = app.oneshot(download_request(zip.job.id)).await.unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(zip_entries(second).await, 2);
}

#[tokio::test]
async fn a_retried_zip_job_has_no_download_while_it_runs() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let zip = finished_zip_job(&state, &token).await;
    let app = wrap_auth(router(state), token);
    let retried = app
        .clone()
        .oneshot(retry_request(zip.job.id))
        .await
        .unwrap();
    assert_eq!(retried.status(), StatusCode::OK);

    let res = app.oneshot(download_request(zip.job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(res).await["message"], "job not complete");
    assert!(!zip.archive.exists());
}

#[tokio::test]
async fn an_archive_built_before_a_retry_is_not_kept() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let zip = finished_zip_job(&state, &token).await;
    let owner = zip.job.user_id;
    build_zip_archive(&state, zip.epoch, owner, zip.job.id)
        .await
        .unwrap();
    confirm_zip_archive(&state, zip.epoch, owner, &zip.job)
        .await
        .unwrap();
    assert!(zip.archive.exists());
    let app = wrap_auth(router(state.clone()), token);
    let retried = app.oneshot(retry_request(zip.job.id)).await.unwrap();
    assert_eq!(retried.status(), StatusCode::OK);
    build_zip_archive(&state, zip.epoch, owner, zip.job.id)
        .await
        .unwrap();

    let confirmed = confirm_zip_archive(&state, zip.epoch, owner, &zip.job).await;

    match confirmed {
        Err(AppError::Conflict(message)) => {
            assert_eq!(message, "job changed while building the archive")
        }
        other => panic!("expected a conflict, got {other:?}"),
    }
    assert!(!zip.archive.exists());
    assert!(zip.dir.join("first.jpg").exists());
}

#[tokio::test]
async fn clearing_jobs_keeps_a_retried_zip_job() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let retried = finished_zip_job(&state, &token).await;
    let finished = JobSeed::new(DOWNLOAD_ZIP_KIND).create(&state, &token).await;
    let finished_dir = zip_job_dir(&state, retried.epoch, finished.user_id, finished.id).unwrap();
    std::fs::create_dir_all(&finished_dir).unwrap();
    complete_next(&state).await;
    let app = wrap_auth(router(state.clone()), token);
    let res = app
        .clone()
        .oneshot(retry_request(retried.job.id))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let cleared = app
        .clone()
        .oneshot(empty_request("DELETE", "/api/jobs"))
        .await
        .unwrap();

    assert_eq!(cleared.status(), StatusCode::NO_CONTENT);
    assert!(retried.dir.join("first.jpg").exists());
    assert!(!finished_dir.exists());
    let listed = body_json(app.oneshot(get("/api/jobs")).await.unwrap()).await;
    let ids: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|job| job["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, [retried.job.id.to_string()]);
}

#[tokio::test]
async fn logging_out_the_retrying_session_cancels_the_reopened_job() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let creator = seed_session(&server, &state).await;
    let retrier = seed_session(&server, &state).await;
    let job = JobSeed::new(DOWNLOAD_ZIP_KIND)
        .create(&state, &creator)
        .await;
    fail_next(&state).await;
    let creator_app = wrap_auth(router(state.clone()), creator);
    let retrier_app = wrap_auth(router(state.clone()), retrier);
    let retried = retrier_app
        .clone()
        .oneshot(retry_request(job.id))
        .await
        .unwrap();
    assert_eq!(retried.status(), StatusCode::OK);

    let logout = empty_request("POST", "/api/auth/logout");
    assert_eq!(
        creator_app.oneshot(logout).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(job_status(&state, job.id).await, JobStatus::Pending);
    let logout = empty_request("POST", "/api/auth/logout");
    assert_eq!(
        retrier_app.oneshot(logout).await.unwrap().status(),
        StatusCode::OK
    );

    assert_eq!(job_status(&state, job.id).await, JobStatus::Cancelled);
    assert!(state.jobs.job_credential(job.id).await.unwrap().is_none());
}
