use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::MockServer;

fn defaults() -> Value {
    json!({
        "mode": "dark",
        "light_theme": "immich-edit-light",
        "dark_theme": "immich-edit-dark",
        "accents": {}
    })
}

async fn read(app: &axum::Router) -> Value {
    let resp = app.clone().oneshot(get("/api/appearance")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

async fn write(app: &axum::Router, body: Value) -> axum::response::Response {
    app.clone()
        .oneshot(json_request("PUT", "/api/appearance", body))
        .await
        .unwrap()
}

#[tokio::test]
async fn appearance_defaults_without_a_stored_row() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    assert_eq!(read(&app).await, defaults());
}

#[tokio::test]
async fn appearance_round_trips_through_put_and_get() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let body = json!({
        "mode": "system",
        "light_theme": "paper",
        "dark_theme": "midnight-2",
        "accents": { "blue": "sky", "green": "mint-3" }
    });

    let resp = write(&app, body.clone()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await, body);
    assert_eq!(read(&app).await, body);
}

#[tokio::test]
async fn appearance_put_replaces_the_previous_value() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let first = json!({
        "mode": "light",
        "light_theme": "paper",
        "dark_theme": "midnight",
        "accents": { "blue": "sky", "red": "rose" }
    });
    let second = json!({
        "mode": "dark",
        "light_theme": "immich-edit-light",
        "dark_theme": "oled",
        "accents": { "green": "mint" }
    });

    assert_eq!(write(&app, first).await.status(), StatusCode::OK);
    assert_eq!(write(&app, second.clone()).await.status(), StatusCode::OK);
    assert_eq!(read(&app).await, second);
}

#[tokio::test]
async fn appearance_bodies_are_validated() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let with = |patch: Value| {
        let mut body = defaults();
        for (key, value) in patch.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    let too_many: serde_json::Map<String, Value> = (0..17)
        .map(|n| (format!("family-{n}"), json!("accent")))
        .collect();

    for (body, want) in [
        (
            with(json!({ "mode": "sepia" })),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            with(json!({ "mode": "Dark" })),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            with(json!({ "extra": true })),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            with(json!({ "light_theme": "Paper" })),
            StatusCode::BAD_REQUEST,
        ),
        (
            with(json!({ "dark_theme": "x".repeat(65) })),
            StatusCode::BAD_REQUEST,
        ),
        (with(json!({ "light_theme": "" })), StatusCode::BAD_REQUEST),
        (
            with(json!({ "dark_theme": "mid night" })),
            StatusCode::BAD_REQUEST,
        ),
        (
            with(json!({ "accents": { "Blue": "sky" } })),
            StatusCode::BAD_REQUEST,
        ),
        (
            with(json!({ "accents": { "blue": "sky_1" } })),
            StatusCode::BAD_REQUEST,
        ),
        (
            with(json!({ "accents": { "blue": "" } })),
            StatusCode::BAD_REQUEST,
        ),
        (
            with(json!({ "accents": too_many })),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let resp = write(&app, body.clone()).await;
        assert_eq!(resp.status(), want, "{body}");
    }
    assert_eq!(read(&app).await, defaults());
}

#[tokio::test]
async fn appearance_accepts_the_limits() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let accents: serde_json::Map<String, Value> = (0..16)
        .map(|n| (format!("family-{n}"), json!("a".repeat(64))))
        .collect();
    let body = json!({
        "mode": "light",
        "light_theme": "z".repeat(64),
        "dark_theme": "0-9",
        "accents": accents
    });

    let resp = write(&app, body.clone()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(read(&app).await, body);
}
