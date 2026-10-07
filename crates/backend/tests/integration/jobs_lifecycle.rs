use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::services::auth_store::AuthKind;
use immich_edit_backend::services::export::{BatchExecutor, DOWNLOAD_ZIP_KIND};
use immich_edit_backend::services::job_runner::JobExecutor;
use immich_edit_backend::services::job_store::{
    JobItemStatus, JobRecord, JobStatus, NewJob, NewJobItem,
};
use immich_edit_backend::state::AppState;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{MockServer, ResponseTemplate};

async fn seed_job(state: &AppState, owner: Uuid, token: &str, kind: &str) -> JobRecord {
    let ctx = state.auth.authenticate(token).await.unwrap().unwrap();
    state
        .jobs
        .create_job(NewJob {
            owner,
            server_epoch: ctx.server_epoch,
            auth_session_id: ctx.session_id,
            kind,
            target: &json!({}),
            params: &json!({}),
            items: &[NewJobItem {
                asset_id: asset_id().to_string(),
                idempotency_key: None,
            }],
            cred: TEST_API_KEY.as_bytes(),
            auth_kind: AuthKind::ApiKey,
        })
        .await
        .unwrap()
}

fn download_request(id: Uuid) -> Request<Body> {
    Request::builder()
        .uri(format!("/api/jobs/{id}/download"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn a_restart_requeues_running_work() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = seed_job(&state, test_user_id(), &token, DOWNLOAD_ZIP_KIND).await;

    let item = state.jobs.claim_next_item().await.unwrap().unwrap();
    assert_eq!(item.status, JobItemStatus::Running);
    assert_eq!(
        state.jobs.get_job(job.id).await.unwrap().unwrap().status,
        JobStatus::Running
    );

    assert_eq!(state.jobs.requeue_running().await.unwrap(), 1);

    assert_eq!(
        state.jobs.get_job(job.id).await.unwrap().unwrap().status,
        JobStatus::Pending
    );
    assert_eq!(
        state.jobs.list_items(job.id).await.unwrap()[0].status,
        JobItemStatus::Pending
    );
    assert!(state.jobs.claim_next_item().await.unwrap().is_some());
}

#[tokio::test]
async fn a_zip_item_reports_metadata_warnings() {
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
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = seed_job(&state, test_user_id(), &token, DOWNLOAD_ZIP_KIND).await;
    let item = state.jobs.claim_next_item().await.unwrap().unwrap();

    let result = BatchExecutor::new(state).execute(job, item).await.unwrap();

    if result["warnings"] != json!(["Metadata not copied: no readable EXIF in the original"]) {
        panic!("result {result}");
    }
}

#[tokio::test]
async fn an_unfinished_zip_job_has_no_download() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = seed_job(&state, test_user_id(), &token, DOWNLOAD_ZIP_KIND).await;
    let app = wrap_auth(router(state), token);

    let res = app.oneshot(download_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(res).await["message"], "job not complete");
}

#[tokio::test]
async fn another_kind_of_job_has_no_download() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let job = seed_job(&state, test_user_id(), &token, "export").await;
    let app = wrap_auth(router(state), token);

    let res = app.oneshot(download_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn another_members_job_is_not_visible() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let admin_token = seed_session(&server, &state).await;
    let member_token = seed_member_session(&server, &state).await;
    let job = seed_job(&state, member_user_id(), &member_token, DOWNLOAD_ZIP_KIND).await;
    let app = wrap_auth(router(state), admin_token);

    let res = app.oneshot(download_request(job.id)).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
