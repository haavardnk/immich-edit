use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

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
