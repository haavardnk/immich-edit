use crate::common::*;
use immich_edit_backend::immich::client::ImmichUser;
use immich_edit_backend::services::auth_store::AuthKind;
use immich_edit_backend::services::job_runner::{JobRunner, UnsupportedExecutor};
use immich_edit_backend::services::job_store::{NewJob, NewJobItem};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tower::ServiceExt;
use tracing::Level;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_rejected_request_logs_one_line_without_the_query() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "message": "nope" })))
        .mount(&server)
        .await;

    let resp = test_app(&server)
        .await
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{}?token=hunter2", asset_id()),
            json!({"isFavorite": true}),
        ))
        .await
        .unwrap();

    if resp.status() != axum::http::StatusCode::UNPROCESSABLE_ENTITY {
        panic!("status {}", resp.status());
    }
    let lines = capture.lines_with("request rejected");
    let [line] = lines.as_slice() else {
        panic!("expected one access line, got {lines:?}");
    };
    for needle in [
        "request_id=",
        &format!("path=\"/api/assets/{}\"", asset_id()),
        "status=422",
        "upstream_rejected",
        "nope",
    ] {
        if !line.contains(needle) {
            panic!("access line misses {needle:?}: {line}");
        }
    }
    if capture.text().contains("hunter2") {
        panic!("query strings must never be logged");
    }
}

#[tokio::test]
async fn a_malformed_body_logs_the_reason() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let req = axum::http::Request::builder()
        .method("PUT")
        .uri(format!("/api/assets/{}", asset_id()))
        .header("content-type", "application/json")
        .body(axum::body::Body::from("{"))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    if resp.status() != axum::http::StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
    let reason = "Failed to parse the request body as JSON";
    let body = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    if !String::from_utf8_lossy(&body).contains(reason) {
        panic!("rejection body lost: {body:?}");
    }
    let lines = capture.lines_with("request rejected");
    let [line] = lines.as_slice() else {
        panic!("expected one access line, got {}", capture.text());
    };
    if !line.contains("status=400") || !line.contains(reason) {
        panic!("access line misses the reason: {line}");
    }
}

#[tokio::test]
async fn an_unreachable_immich_logs_the_transport_cause() {
    let (capture, _guard) = capture_json_logs(Level::WARN);
    let server = MockServer::builder().start().await;
    let app = test_app(&server).await;
    drop(server);

    let resp = app.oneshot(get("/api/albums")).await.unwrap();

    if resp.status() != axum::http::StatusCode::BAD_GATEWAY {
        panic!("status {}", resp.status());
    }
    let text = capture.text();
    let events: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let Some(cause) = events
        .iter()
        .find(|e| e["message"] == "immich request failed")
        .and_then(|e| e["error"].as_str())
    else {
        panic!("immich failure not logged: {text}");
    };
    if cause.starts_with('"') || !cause.to_lowercase().contains("connection refused") {
        panic!("immich failure lost its cause: {cause}");
    }
    if !events.iter().any(|e| {
        e["message"] == "request failed"
            && e["status"] == 502
            && e["code"] == "upstream_unavailable"
    }) {
        panic!("access line missing: {text}");
    }
}

#[tokio::test]
async fn an_unreadable_session_is_logged() {
    let (capture, _guard) = capture_logs(Level::ERROR);
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let user = ImmichUser {
        id: test_user_id(),
        email: "admin@test.local".into(),
        name: "Admin".into(),
        is_admin: true,
    };
    let token =
        seed_session_with_cred(&server, &state, user, AuthKind::ApiKey, &[0xff, 0xfe]).await;

    let resp = wrap_auth(router(state), token)
        .oneshot(get("/api/albums"))
        .await
        .unwrap();

    if resp.status() != axum::http::StatusCode::UNAUTHORIZED {
        panic!("status {}", resp.status());
    }
    let lines = capture.lines_with("stored immich credential is not valid utf-8");
    let [line] = lines.as_slice() else {
        panic!("expected one error line, got {}", capture.text());
    };
    if !line.contains("request_id=") || !line.contains("session=") {
        panic!("error line lacks context: {line}");
    }
}

#[tokio::test]
async fn a_failed_job_item_is_logged_with_its_job() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let ctx = state.auth.authenticate(&token).await.unwrap().unwrap();
    let job = state
        .jobs
        .create_job(NewJob {
            owner: test_user_id(),
            server_epoch: ctx.server_epoch,
            auth_session_id: ctx.session_id,
            kind: "mystery",
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
        .unwrap();

    let (stop, stopped) = watch::channel(false);
    let runner = tokio::spawn(
        JobRunner::new(state.jobs.clone(), Arc::new(UnsupportedExecutor), 1).run(stopped),
    );
    for _ in 0..50 {
        if state.jobs.get_job(job.id).await.unwrap().unwrap().failed == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let _ = stop.send(true);
    let _ = runner.await;

    let lines = capture.lines_with("job item failed");
    let [line] = lines.as_slice() else {
        panic!("expected one failure line, got {}", capture.text());
    };
    for needle in [
        &format!("job_id={}", job.id),
        "kind=\"mystery\"",
        "unsupported job kind: mystery",
    ] {
        if !line.contains(needle) {
            panic!("failure line misses {needle:?}: {line}");
        }
    }
}
