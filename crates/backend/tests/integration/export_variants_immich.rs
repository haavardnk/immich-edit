use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::routes::export::{
    ExportParams, ExportToImmichBody, StackPrimary, hash_request,
};
use immich_edit_backend::services::edits_store::{ExportJobKey, ExportJobStatus};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn web_params() -> Value {
    json!({ "filename_template": "{name}_{preset}", "metadata": "none" })
}

fn print_params() -> Value {
    json!({ "filename_template": "{name}_{preset}", "metadata": "none", "format": "png" })
}

fn web_and_print() -> Value {
    json!({ "variants": [
        preset_variant("Web", web_params()),
        preset_variant("Print", print_params()),
    ] })
}

async fn mock_upload(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/assets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": Uuid::new_v4(),
            "status": "created"
        })))
        .mount(server)
        .await;
}

async fn uploads(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == "POST" && r.url.path() == "/api/assets")
        .count()
}

fn variant_key(key: &str) -> ExportJobKey<'_> {
    ExportJobKey {
        owner: test_user_id(),
        asset_id: AssetKey::master(asset_id()),
        key,
    }
}

async fn immich_variant_job(
    server: &MockServer,
) -> (immich_edit_backend::state::AppState, axum::Router, Uuid) {
    mock_named_asset(server, asset_id(), "DSC0001.ARW").await;
    mock_upload(server).await;
    let state = test_state(server).await;
    let token = seed_session(server, &state).await;
    let app = wrap_auth(router(state.clone()), token);
    let job_id = created_job_id(
        app.clone()
            .oneshot(create_job_request(
                "export_immich",
                web_and_print(),
                &[asset_id().to_string()],
            ))
            .await
            .unwrap(),
    )
    .await;
    (state, app, job_id)
}

#[tokio::test]
async fn an_immich_job_keys_each_preset_of_a_photo_apart() {
    let server = MockServer::start().await;
    let (state, _, job_id) = immich_variant_job(&server).await;

    let results = run_all(&state).await;

    let filenames: Vec<(&str, &str)> = results
        .iter()
        .map(|result| {
            (
                result["preset"].as_str().unwrap_or_default(),
                result["filename"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        filenames,
        [("Web", "DSC0001_Web.jpg"), ("Print", "DSC0001_Print.png")]
    );
    for key in [format!("job-{job_id}"), format!("job-{job_id}-v1")] {
        let record = state.edits.get_export_job(variant_key(&key)).await.unwrap();
        assert!(record.is_some(), "{key}");
    }
    assert_eq!(uploads(&server).await, 2);
}

#[tokio::test]
async fn a_retried_preset_resumes_its_own_uploaded_export() {
    let server = MockServer::start().await;
    let (state, app, job_id) = immich_variant_job(&server).await;
    complete_next(&state).await;
    let failed = fail_next(&state).await;
    let key = format!("job-{job_id}-v1");
    let uploaded = Uuid::new_v4();
    let body = ExportToImmichBody {
        edits: Default::default(),
        params: serde_json::from_value::<ExportParams>(print_params()).unwrap(),
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
        replace_original: false,
    };
    state
        .edits
        .put_export_job_uploaded(
            variant_key(&key),
            &hash_request(AssetKey::master(asset_id()), &body),
            uploaded,
            "DSC0001_Print.png",
            "created",
            &[],
        )
        .await
        .unwrap();

    let resp = app.oneshot(retry_request(job_id)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let item = state.jobs.claim_next_item().await.unwrap().unwrap();
    assert_eq!((item.id, item.variant), (failed.id, 1));
    let result = run_item(&state, item).await;

    assert_eq!(result["asset_id"], uploaded.to_string());
    assert_eq!(result["filename"], "DSC0001_Print.png");
    assert_eq!(result["preset"], "Print");
    assert_eq!(uploads(&server).await, 0);
    let record = state
        .edits
        .get_export_job(variant_key(&key))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.status, ExportJobStatus::Completed);
}
