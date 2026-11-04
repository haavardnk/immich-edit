use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::MockServer;

fn defaults() -> Value {
    json!({
        "click_target": "editor",
        "pinned_settings": null,
        "mask_overlay_color": null,
        "shortcuts": {},
        "desktop_sections": null,
        "touch_subviews": null
    })
}

async fn read(app: &axum::Router) -> Value {
    let resp = app.clone().oneshot(get("/api/preferences")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

async fn patch(app: &axum::Router, body: Value) -> axum::response::Response {
    app.clone()
        .oneshot(json_request("PATCH", "/api/preferences", body))
        .await
        .unwrap()
}

async fn patch_ok(app: &axum::Router, body: Value) -> Value {
    let resp = patch(app, body.clone()).await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    body_json(resp).await
}

fn function_keys(count: usize) -> Vec<String> {
    (1..=count).map(|n| format!("F{n}")).collect()
}

fn with(changes: Value) -> Value {
    let mut merged = defaults();
    for (key, value) in changes.as_object().unwrap() {
        merged[key] = value.clone();
    }
    merged
}

#[tokio::test]
async fn preferences_default_without_a_stored_row() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    assert_eq!(read(&app).await, defaults());
}

#[tokio::test]
async fn preferences_round_trip_every_field() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let all = json!({
        "click_target": "loupe",
        "pinned_settings": ["exposure", "white_balance", "color-grading"],
        "mask_overlay_color": "#ff8800",
        "shortcuts": {
            "help": ["?", "Shift+/"],
            "backToGrid": ["Escape"],
            "editor.copy": ["Mod+Shift+c"],
            "compare.focusCycle": []
        },
        "desktop_sections": [{ "id": "basic", "hidden": false }, { "id": "detail", "hidden": true }],
        "touch_subviews": [{ "id": "light", "hidden": true }]
    });

    assert_eq!(patch_ok(&app, all.clone()).await, all);
    assert_eq!(read(&app).await, all);
}

#[tokio::test]
async fn preferences_patches_merge_without_clobbering() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let first = patch_ok(&app, json!({ "click_target": "loupe" })).await;
    assert_eq!(first, with(json!({ "click_target": "loupe" })));

    let second = patch_ok(&app, json!({ "shortcuts": { "help": ["F1"] } })).await;
    let both = with(json!({
        "click_target": "loupe",
        "shortcuts": { "help": ["F1"] }
    }));
    assert_eq!(second, both);
    assert_eq!(read(&app).await, both);

    assert_eq!(patch_ok(&app, json!({})).await, both);
}

#[tokio::test]
async fn preferences_null_resets_a_nullable_field() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    patch_ok(
        &app,
        json!({
            "pinned_settings": ["exposure"],
            "mask_overlay_color": "#00ff00"
        }),
    )
    .await;

    let reset = patch_ok(&app, json!({ "pinned_settings": null })).await;
    assert_eq!(reset, with(json!({ "mask_overlay_color": "#00ff00" })));
    assert_eq!(read(&app).await, reset);
}

#[tokio::test]
async fn preferences_validation_failures_are_bad_requests() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    for (body, field) in [
        (json!({ "mask_overlay_color": "red" }), "mask_overlay_color"),
        (
            json!({ "shortcuts": { "loupeNav": ["j", "k", "j"] } }),
            "\"loupeNav\" lists \"j\" twice",
        ),
    ] {
        let resp = patch(&app, body.clone()).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
        let error = body_json(resp).await;
        assert_eq!(error["code"], "bad_request", "{body}");
        let message = error["message"].as_str().unwrap();
        assert!(message.contains(field), "{body} -> {message}");
    }
    assert_eq!(read(&app).await, defaults());
}

#[tokio::test]
async fn preferences_a_rejected_patch_keeps_the_stored_record() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let stored = patch_ok(&app, json!({ "click_target": "loupe" })).await;

    let resp = patch(
        &app,
        json!({ "click_target": "editor", "mask_overlay_color": "#XYZ" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(read(&app).await, stored);
}

#[tokio::test]
async fn preferences_a_shape_error_is_unprocessable() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let resp = patch(&app, json!({ "theme": "dark" })).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(read(&app).await, defaults());
}

#[tokio::test]
async fn preferences_accept_the_limits() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let ids: Vec<String> = (0..64).map(|n| format!("s{n}")).collect();
    let listed: Vec<Value> = ids
        .iter()
        .map(|id| json!({ "id": id, "hidden": true }))
        .collect();
    let mut shortcuts: serde_json::Map<String, Value> =
        (1..256).map(|n| (format!("a{n}"), json!([]))).collect();
    let mut keys = vec![
        "Mod+Alt+Shift+z".to_string(),
        "+".into(),
        "~".into(),
        format!("K{}", "x".repeat(31)),
    ];
    keys.extend(function_keys(12));
    shortcuts.insert("a0".into(), json!(keys));
    let body = json!({
        "click_target": "editor",
        "pinned_settings": ids,
        "mask_overlay_color": "#000000",
        "shortcuts": shortcuts,
        "desktop_sections": listed,
        "touch_subviews": []
    });

    assert_eq!(patch_ok(&app, body.clone()).await, body);
    assert_eq!(read(&app).await, body);
}
