use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::services::edits_store::EditWrite;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

use crate::common::{
    body_json, router, seed_member_session, seed_session, test_app, test_state, test_user_id,
    wrap_auth,
};

#[cfg(feature = "ml")]
use crate::common::json_request;
#[cfg(feature = "ml")]
use serde_json::Value;

fn cube(color: f32) -> Vec<u8> {
    let mut src = String::from("LUT_3D_SIZE 2\n");
    for i in 0..8 {
        let v = if i == 7 { color } else { 0.0 };
        src.push_str(&format!("{v} {v} {v}\n"));
    }
    src.into_bytes()
}

fn lut_request(name: &str, body: Vec<u8>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/luts?name={name}"))
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn a_thumb_for_an_unedited_asset_is_not_found() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = Uuid::new_v4();
    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/edited-thumb?h=abc&size=400"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_thumb_hash_that_does_not_match_the_edits_is_not_found() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let id = AssetKey::master(Uuid::new_v4());
    state
        .edits
        .put(test_user_id(), id, EditWrite::default())
        .await
        .unwrap();
    let app = wrap_auth(router(state), token);
    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/edited-thumb?h=not-the-hash"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_lut_import_round_trips_and_rejects_a_duplicate() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .clone()
        .oneshot(lut_request("Warm", cube(1.0)))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created = body_json(res).await;
    assert_eq!(created["name"], "Warm");
    assert_eq!(created["lut_size"], 2);

    let res = app.oneshot(lut_request("Other", cube(1.0))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = body_json(res).await;
    assert_eq!(
        body["message"],
        format!("lut already exists: {}", created["id"].as_str().unwrap())
    );
}

#[tokio::test]
async fn a_lut_import_rejects_a_blank_name_and_a_broken_cube() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .clone()
        .oneshot(lut_request("%20", cube(0.5)))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res).await;
    assert_eq!(body["message"], "name is empty");

    let res = app
        .oneshot(lut_request("Broken", b"LUT_3D_SIZE 2\n0 0 0\n".to_vec()))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_member_cannot_import_a_lut() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_member_session(&server, &state).await;
    let app = wrap_auth(router(state), token);
    let res = app.oneshot(lut_request("Warm", cube(1.0))).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

fn bundled_dcp() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/dcp/Canon EOS 5D.dcp");
    std::fs::read(path).unwrap()
}

fn dcp_request(query: &str, body: Vec<u8>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/dcp?{query}"))
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn a_dcp_import_round_trips_and_rejects_a_duplicate() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .clone()
        .oneshot(dcp_request("name=Probe&camera=ILCE-7M4", bundled_dcp()))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created = body_json(res).await;
    assert_eq!(created["name"], "Probe");

    let res = app
        .oneshot(dcp_request("name=Again", bundled_dcp()))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = body_json(res).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains(created["id"].as_str().unwrap()),
        "{body}"
    );
}

#[tokio::test]
async fn a_dcp_import_rejects_bytes_that_are_not_a_profile() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .oneshot(dcp_request("name=Broken", b"not a dcp".to_vec()))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_member_cannot_import_a_dcp() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let token = seed_member_session(&server, &state).await;
    let app = wrap_auth(router(state), token);
    let res = app
        .oneshot(dcp_request("name=Probe", bundled_dcp()))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn bake_parameters_outside_their_range_are_rejected() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/masks/rebake",
            serde_json::json!({
                "asset_id": Uuid::new_v4(),
                "prob_raster_id": "missing",
                "grow": 1.0e9,
            }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .starts_with("grow out of range"),
        "{body}"
    );

    let res = app
        .oneshot(json_request(
            "POST",
            "/api/masks/rebake",
            serde_json::json!({
                "asset_id": Uuid::new_v4(),
                "prob_raster_id": "missing",
                "range": { "min": -1.0, "max": 2.0 },
            }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res).await;
    assert_eq!(body["message"], "range must be within 0..1");
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn mask_generation_is_rejected_when_segmentation_is_off() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = Uuid::new_v4();
    let res = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/masks/generate"),
            serde_json::json!({ "kind": "sky" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res).await;
    assert_eq!(body["message"], "segmentation is disabled on this server");
}

#[cfg(feature = "ml")]
async fn click_error(bbox: Value) -> Value {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = Uuid::new_v4();
    let res = app
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{id}/masks/click"),
            serde_json::json!({ "bbox": bbox }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    body_json(res).await
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn a_click_box_outside_the_frame_is_rejected() {
    let body =
        click_error(serde_json::json!({ "x0": -0.1, "y0": 0.2, "x1": 0.6, "y1": 0.8 })).await;
    assert_eq!(body["message"], "box coordinates must be within 0..1");
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn a_click_box_thinner_than_the_minimum_is_rejected() {
    let body =
        click_error(serde_json::json!({ "x0": 0.5, "y0": 0.2, "x1": 0.502, "y1": 0.8 })).await;
    assert_eq!(body["message"], "box is too small");
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn a_valid_click_box_reaches_the_segmentation_check() {
    let body = click_error(serde_json::json!({ "x0": 0.8, "y0": 0.8, "x1": 0.2, "y1": 0.2 })).await;
    assert_eq!(body["message"], "segmentation is disabled on this server");
}
