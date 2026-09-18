mod common;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::json;
use std::net::SocketAddr;
use tower::ServiceExt;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROXY: &str = "127.0.0.1:9000";

async fn configured_app(server: &MockServer) -> axum::Router {
    let state = test_state(server).await;
    state.instance.claim(&server.uri()).await.unwrap();
    router(state)
}

async fn mock_password_login(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .and(body_partial_json(json!({ "password": "right" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "accessToken": "tok-abc",
            "userId": test_user_id(),
            "userEmail": "victim@test.local",
            "name": "Victim",
            "isAdmin": false,
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .and(body_partial_json(json!({ "password": "wrong" })))
        .respond_with(ResponseTemplate::new(401))
        .mount(server)
        .await;
}

async fn attempt(app: &axum::Router, client_ip: &str, email: &str, password: &str) -> StatusCode {
    let body = json!({ "email": email, "password": password });
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login/password")
        .header("content-type", "application/json")
        .header("x-forwarded-for", client_ip)
        .extension(ConnectInfo::<SocketAddr>(PROXY.parse().unwrap()))
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn five_failures_lock_the_client_and_answer_with_retry_after() {
    let server = MockServer::start().await;
    mock_password_login(&server).await;
    let app = configured_app(&server).await;

    for n in 1..=5 {
        let got = attempt(&app, "203.0.113.5", "victim@test.local", "wrong").await;
        if got != StatusCode::UNAUTHORIZED {
            panic!("attempt {n}: expected 401, got {got}");
        }
    }

    let body = json!({ "email": "victim@test.local", "password": "wrong" });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login/password")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "203.0.113.5")
                .extension(ConnectInfo::<SocketAddr>(PROXY.parse().unwrap()))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::TOO_MANY_REQUESTS {
        panic!("sixth attempt: expected 429, got {}", resp.status());
    }
    let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let Some(secs) = retry_after else {
        panic!("429 must carry a numeric Retry-After");
    };
    if secs == 0 || secs > 15 * 60 {
        panic!("Retry-After {secs} is outside the lockout window");
    }

    let other = attempt(&app, "203.0.113.6", "victim@test.local", "wrong").await;
    if other != StatusCode::UNAUTHORIZED {
        panic!("a different client must still be served, got {other}");
    }
}

#[tokio::test]
async fn a_successful_login_resets_the_client_counter() {
    let server = MockServer::start().await;
    mock_password_login(&server).await;
    let app = configured_app(&server).await;

    for _ in 0..4 {
        attempt(&app, "203.0.113.7", "victim@test.local", "wrong").await;
    }
    let ok = attempt(&app, "203.0.113.7", "victim@test.local", "right").await;
    if ok != StatusCode::OK {
        panic!("the correct password must be accepted, got {ok}");
    }
    for n in 1..=4 {
        let got = attempt(&app, "203.0.113.7", "victim@test.local", "wrong").await;
        if got != StatusCode::UNAUTHORIZED {
            panic!("post-reset attempt {n}: expected 401, got {got}");
        }
    }
}

#[tokio::test]
async fn rotating_clients_still_lock_the_identity() {
    let server = MockServer::start().await;
    mock_password_login(&server).await;
    let app = configured_app(&server).await;

    for n in 0..25 {
        let ip = format!("203.0.113.{n}");
        let got = attempt(&app, &ip, "victim@test.local", "wrong").await;
        if got != StatusCode::UNAUTHORIZED {
            panic!("failure {n} from {ip}: expected 401, got {got}");
        }
    }

    let locked = attempt(&app, "198.51.100.1", "victim@test.local", "wrong").await;
    if locked != StatusCode::TOO_MANY_REQUESTS {
        panic!("a fresh client must inherit the identity lock, got {locked}");
    }
    let other = attempt(&app, "198.51.100.1", "bystander@test.local", "wrong").await;
    if other != StatusCode::UNAUTHORIZED {
        panic!("an unrelated identity must be unaffected, got {other}");
    }
}
