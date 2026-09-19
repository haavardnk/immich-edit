mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = http_body_util::BodyExt::collect(res.into_body())
        .await
        .unwrap()
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn mock_me(server: &MockServer, id: Uuid, is_admin: bool) {
    Mock::given(method("GET"))
        .and(path("/api/users/me"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "email": "admin@test.local",
            "name": "Admin",
            "isAdmin": is_admin,
        })))
        .mount(server)
        .await;
}

fn access_request(id: Uuid, enabled: bool) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(format!("/api/admin/users/{id}/access"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "enabled": enabled }).to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn an_admin_cannot_disable_their_own_account() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(access_request(test_user_id(), false))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(res).await["message"],
        "cannot disable your own account"
    );
}

#[tokio::test]
async fn access_changes_for_an_unknown_user_are_not_found() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(access_request(Uuid::new_v4(), false))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn disabling_a_member_revokes_their_sessions() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let state = test_state(&server).await;
    let admin_token = seed_session(&server, &state).await;
    seed_member_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), admin_token);

    let res = app
        .oneshot(access_request(member_user_id(), false))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        state
            .auth
            .list_sessions(member_user_id())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        !state
            .auth
            .get_user(member_user_id())
            .await
            .unwrap()
            .unwrap()
            .access_enabled
    );
}

#[tokio::test]
async fn a_stale_admin_session_is_rejected() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), false).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(access_request(member_user_id(), false))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn purging_an_unknown_user_is_not_found() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/users/{}/data", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_rebind_needs_a_matching_confirmation_hostname() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/instance/rebind")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "immich_url": "http://192.168.1.10:2283",
                        "confirm_hostname": "192.168.1.11",
                        "api_key": "other-key",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(res).await["message"],
        "confirmation hostname does not match"
    );
}

#[tokio::test]
async fn a_rebind_to_a_blocked_url_is_rejected() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/instance/rebind")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "immich_url": "http://169.254.169.254",
                        "confirm_hostname": "169.254.169.254",
                        "api_key": "other-key",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
