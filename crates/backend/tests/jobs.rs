mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

#[tokio::test]
async fn create_job_rejects_invalid_requests() {
    for (body, message) in [
        (
            json!({
                "kind": "apply_preset",
                "target": { "search": { "albumIds": [album_id()] } },
                "params": {}
            }),
            None,
        ),
        (
            json!({
                "kind": "not_a_real_kind",
                "asset_ids": [Uuid::new_v4()],
                "params": {}
            }),
            Some("unknown job kind"),
        ),
    ] {
        let server = MockServer::start().await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(json_request("POST", "/api/jobs", body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
        let text = String::from_utf8(body_bytes(resp).await).unwrap();
        if let Some(message) = message {
            assert!(text.contains(message), "{text}");
        }
    }
}

#[tokio::test]
async fn create_edit_jobs_create_items_for_ids() {
    for (kind, params) in [
        (
            "paste_edits",
            json!({ "manifest": { "schema_version": 3, "ops": {} } }),
        ),
        ("reset_edits", json!({})),
    ] {
        let server = MockServer::start().await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(json_request(
                "POST",
                "/api/jobs",
                json!({
                    "kind": kind,
                    "asset_ids": [Uuid::new_v4(), Uuid::new_v4()],
                    "params": params
                }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{kind}");
        let job = body_json(resp).await;
        assert_eq!(job["kind"], kind);
        assert_eq!(job["total"], 2);
    }
}
