use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::state::AppState;
use tower::ServiceExt;
use wiremock::MockServer;

fn upload(bytes: Vec<u8>) -> Request<Body> {
    Request::post("/api/rasters?width=4&height=3")
        .header("content-type", "application/octet-stream")
        .body(Body::from(bytes))
        .unwrap()
}

#[tokio::test]
async fn raster_upload_get_roundtrip() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let bytes = vec![0xABu8; 4 * 3];
    let resp = app.clone().oneshot(upload(bytes.clone())).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    let raster_id = v["raster_id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(get(&format!("/api/rasters/{raster_id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("x-raster-width").unwrap(), "4");
    assert_eq!(resp.headers().get("x-raster-height").unwrap(), "3");
    let got = body_bytes(resp).await;
    assert_eq!(got, bytes);
}

#[tokio::test]
async fn raster_upload_rejects_bad_size() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app.oneshot(upload(vec![0u8; 5])).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

async fn raster_ref_count(state: &AppState) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM raster_refs")
        .fetch_one(&state.edits.pool())
        .await
        .unwrap()
}

#[tokio::test]
async fn a_rebind_clears_raster_refs() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    seed_session(&server, &state).await;
    for owner in [test_user_id(), member_user_id()] {
        sqlx::query("INSERT INTO raster_refs (user_id, asset_id, raster_id) VALUES (?1, ?2, ?3)")
            .bind(owner.to_string())
            .bind(asset_id().to_string())
            .bind("a".repeat(64))
            .execute(&state.edits.pool())
            .await
            .unwrap();
    }
    assert_eq!(raster_ref_count(&state).await, 2);

    rebind_instance(&server, &state).await;

    assert_eq!(raster_ref_count(&state).await, 0);
}
