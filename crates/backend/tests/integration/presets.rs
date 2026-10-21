use crate::common::*;
use axum::http::{HeaderValue, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

fn jpeg_long_edge(jpeg: &[u8]) -> u16 {
    let mut at = 2;
    while jpeg[at + 1] != 0xc0 {
        at += 2 + usize::from(u16::from_be_bytes([jpeg[at + 2], jpeg[at + 3]]));
    }
    let height = u16::from_be_bytes([jpeg[at + 5], jpeg[at + 6]]);
    let width = u16::from_be_bytes([jpeg[at + 7], jpeg[at + 8]]);
    height.max(width)
}

fn preset_body(name: &str, group: Option<&str>) -> Value {
    let mut body = json!({
        "name": name,
        "manifest": { "schema_version": 2, "ops": { "exposure": { "ev": 1.5 } } },
    });
    if let Some(group) = group {
        body["group_name"] = json!(group);
    }
    body
}

#[tokio::test]
async fn a_preset_survives_create_read_update_and_delete() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/presets",
            preset_body("Portrait", Some(" Studio ")),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("create status {}", resp.status());
    }
    let created = body_json(resp).await;
    let id = created["id"].as_str().unwrap().to_string();
    if created["group_name"] != json!("Studio") {
        panic!("the group name must be trimmed: {created}");
    }

    let resp = app
        .clone()
        .oneshot(get(&format!("/api/presets/{id}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("get status {}", resp.status());
    }

    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/presets/{id}"),
            preset_body("Landscape", None),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("update status {}", resp.status());
    }
    let updated = body_json(resp).await;
    if updated["name"] != json!("Landscape") || updated["group_name"] != Value::Null {
        panic!("the update must replace both fields: {updated}");
    }

    let resp = app.clone().oneshot(get("/api/presets")).await.unwrap();
    let listed = body_json(resp).await;
    if listed.as_array().map(|a| a.len()) != Some(1) {
        panic!("the update must not add a second preset: {listed}");
    }

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/presets/{id}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }

    let resp = app
        .oneshot(get(&format!("/api/presets/{id}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("a deleted preset must be gone, got {}", resp.status());
    }
}

#[tokio::test]
async fn a_preset_name_or_group_outside_the_limits_is_rejected() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let long_name = "n".repeat(81);
    let long_group = "g".repeat(61);

    for body in [
        preset_body("", None),
        preset_body("   ", None),
        preset_body(&long_name, None),
        preset_body("Portrait", Some(&long_group)),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/presets", body.clone()))
            .await
            .unwrap();
        if resp.status() != StatusCode::BAD_REQUEST {
            panic!("{body}: expected 400, got {}", resp.status());
        }
    }

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/presets",
            preset_body(&"n".repeat(80), Some(&"g".repeat(60))),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("the limits themselves must pass, got {}", resp.status());
    }
}

#[tokio::test]
async fn another_members_preset_id_is_not_found() {
    let server = MockServer::start().await;
    let (owner, member) = two_owner_apps(&server).await;

    let resp = owner
        .oneshot(json_request(
            "POST",
            "/api/presets",
            preset_body("Portrait", None),
        ))
        .await
        .unwrap();
    let id = body_json(resp).await["id"].as_str().unwrap().to_string();

    for request in [
        get(&format!("/api/presets/{id}")),
        json_request(
            "PUT",
            &format!("/api/presets/{id}"),
            preset_body("Stolen", None),
        ),
        empty_request("DELETE", &format!("/api/presets/{id}")),
    ] {
        let method = request.method().clone();
        let resp = member.clone().oneshot(request).await.unwrap();
        if resp.status() != StatusCode::NOT_FOUND {
            panic!("{method}: expected 404, got {}", resp.status());
        }
    }

    let unknown = Uuid::new_v4();
    let resp = member
        .oneshot(get(&format!("/api/presets/{unknown}")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("an unknown id must be 404, got {}", resp.status());
    }
}

#[tokio::test]
async fn a_preset_thumb_is_capped_and_revalidates_until_the_preset_changes() {
    let server = MockServer::start().await;
    let asset = asset_id();
    mock_original(&server, asset).await;
    let app = test_app(&server).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/presets",
            preset_body("Portrait", None),
        ))
        .await
        .unwrap();
    let id = body_json(resp).await["id"].as_str().unwrap().to_string();
    let uri = format!("/api/assets/{asset}/presets/{id}/thumb?max=4096");
    let revalidate = |etag: &str| {
        let mut req = get(&uri);
        req.headers_mut()
            .insert("if-none-match", HeaderValue::from_str(etag).unwrap());
        req
    };

    let resp = app.clone().oneshot(get(&uri)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("first status {}", resp.status());
    }
    let etag = header_str(&resp, "etag").expect("etag");
    let cache_control = header_str(&resp, "cache-control").unwrap_or_default();
    if !cache_control.contains("must-revalidate") {
        panic!("cache-control: {cache_control}");
    }
    let long_edge = jpeg_long_edge(&body_bytes(resp).await);
    if long_edge == 0 || long_edge > 256 {
        panic!("the thumb must be capped at 256 px, got {long_edge}");
    }

    let resp = app.clone().oneshot(revalidate(&etag)).await.unwrap();
    if resp.status() != StatusCode::NOT_MODIFIED {
        panic!("expected 304, got {}", resp.status());
    }

    let mut changed = preset_body("Portrait", None);
    changed["manifest"]["ops"]["exposure"]["ev"] = json!(-1.0);
    app.clone()
        .oneshot(json_request("PUT", &format!("/api/presets/{id}"), changed))
        .await
        .unwrap();
    let resp = app.oneshot(revalidate(&etag)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("a changed preset must re-render, got {}", resp.status());
    }
}
