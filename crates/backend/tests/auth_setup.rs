mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn to_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn cookie_from(resp: &axum::response::Response) -> String {
    let raw = resp
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    raw.split(';').next().unwrap().to_string()
}

async fn mock_login(server: &MockServer, admin: bool) -> Uuid {
    let uid = Uuid::new_v4();
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "accessToken": "tok-abc",
            "userId": uid,
            "userEmail": "a@b.test",
            "name": "A",
            "isAdmin": admin,
        })))
        .mount(server)
        .await;
    uid
}

#[tokio::test]
async fn setup_then_me_flow() {
    let server = MockServer::start().await;
    mock_login(&server, true).await;
    let state = test_state(&server).await;
    let app = router(state);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(to_json(resp).await["configured"], json!(false));

    let setup_body = json!({"immich_url": server.uri(), "email": "a@b.test", "password": "pw"});
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup/complete")
                .header("content-type", "application/json")
                .body(Body::from(setup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let cookie = cookie_from(&resp);
    assert!(cookie.starts_with("immich_edit_auth="));
    assert_eq!(to_json(resp).await["is_admin"], json!(true));

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(to_json(resp).await["configured"], json!(true));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(to_json(resp).await["email"], json!("a@b.test"));
}

#[tokio::test]
async fn setup_rejects_non_admin() {
    let server = MockServer::start().await;
    mock_login(&server, false).await;
    let state = test_state(&server).await;
    let app = router(state);

    let setup_body = json!({"immich_url": server.uri(), "email": "a@b.test", "password": "pw"});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup/complete")
                .header("content-type", "application/json")
                .body(Body::from(setup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert_eq!(to_json(resp).await["code"], json!("admin_required"));
}

#[tokio::test]
async fn me_requires_session() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
