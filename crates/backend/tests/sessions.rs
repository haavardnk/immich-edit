mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use immich_edit_backend::immich::client::ImmichUser;
use immich_edit_backend::services::auth_store::AuthKind;
use immich_edit_backend::state::AppState;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = http_body_util::BodyExt::collect(res.into_body())
        .await
        .unwrap()
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn admin_user() -> ImmichUser {
    ImmichUser {
        id: test_user_id(),
        email: "admin@test.local".into(),
        name: "Admin".into(),
        is_admin: true,
    }
}

async fn second_admin_session(server: &MockServer, state: &AppState) -> String {
    seed_session_with_cred(
        server,
        state,
        admin_user(),
        AuthKind::ApiKey,
        TEST_API_KEY.as_bytes(),
    )
    .await
}

async fn list_sessions(app: axum::Router) -> Value {
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    body_json(res).await
}

#[tokio::test]
async fn a_session_list_marks_only_the_calling_session_as_current() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = second_admin_session(&server, &state).await;
    second_admin_session(&server, &state).await;

    let body = list_sessions(wrap_auth(router(state), token)).await;
    let sessions = body["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    let current = sessions
        .iter()
        .filter(|s| s["current"] == Value::Bool(true))
        .count();
    assert_eq!(current, 1);
}

#[tokio::test]
async fn another_users_session_id_is_not_found() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = second_admin_session(&server, &state).await;
    seed_member_session(&server, &state).await;
    let member_session = state
        .auth
        .list_sessions(member_user_id())
        .await
        .unwrap()
        .remove(0);
    let app = wrap_auth(router(state.clone()), token);

    for id in [member_session.id, Uuid::new_v4()] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/auth/sessions/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{id}");
    }
    assert_eq!(
        state
            .auth
            .list_sessions(member_user_id())
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn revoking_all_sessions_keeps_the_calling_one() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = second_admin_session(&server, &state).await;
    second_admin_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/sessions/revoke-all")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = list_sessions(app).await;
    assert_eq!(body["sessions"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_revoked_session_stops_authenticating() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let keeper = second_admin_session(&server, &state).await;
    let doomed = second_admin_session(&server, &state).await;
    let doomed_id = state
        .auth
        .authenticate(&doomed)
        .await
        .unwrap()
        .unwrap()
        .session_id;

    let res = wrap_auth(router(state.clone()), keeper)
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/auth/sessions/{doomed_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = wrap_auth(router(state), doomed)
        .oneshot(
            Request::builder()
                .uri("/api/auth/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
