use crate::common::*;
use axum::http::StatusCode;
use tower::ServiceExt;
use wiremock::MockServer;

#[tokio::test]
async fn cached_preview_frame_is_not_served_to_another_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original_owned_by_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let body = serde_json::json!({"max_edge": 512, "edits": {}});
    let resp = admin
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            body.clone(),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("owner preview status {}", resp.status());
    }

    let resp = member
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/preview"),
            body,
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_GATEWAY {
        panic!("member preview status {}", resp.status());
    }
}

#[tokio::test]
async fn cached_quality_frame_is_not_served_to_another_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original_owned_by_admin(&server, id).await;
    mock_asset_detail(&server).await;
    let (admin, member) = two_owner_apps(&server).await;

    let body = serde_json::json!({"edits": {}});
    let resp = admin
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/export"),
            body.clone(),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("owner export status {}", resp.status());
    }

    let resp = member
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/export"),
            body,
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_GATEWAY {
        panic!("member export status {}", resp.status());
    }
}
