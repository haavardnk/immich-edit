mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use immich_edit_backend::asset_key::AssetKey;
use raw_pipeline::edit_manifest::EditManifest;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

use common::{seed_member_session, test_app, test_state, wrap_auth};

async fn body_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

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
    let token = common::seed_session(&server, &state).await;
    let id = AssetKey::master(Uuid::new_v4());
    state
        .edits
        .put(
            common::test_user_id(),
            id,
            EditManifest::default(),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    let app = wrap_auth(common::router(state), token);
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
    let created = body_json(res.into_body()).await;
    assert_eq!(created["name"], "Warm");
    assert_eq!(created["lut_size"], 2);

    let res = app.oneshot(lut_request("Other", cube(1.0))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = body_json(res.into_body()).await;
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
    let body = body_json(res.into_body()).await;
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
    let app = wrap_auth(common::router(state), token);
    let res = app.oneshot(lut_request("Warm", cube(1.0))).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[cfg(feature = "ml")]
fn rebake_request(body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/masks/rebake")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn bake_parameters_outside_their_range_are_rejected() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let res = app
        .clone()
        .oneshot(rebake_request(serde_json::json!({
            "asset_id": Uuid::new_v4(),
            "prob_raster_id": "missing",
            "grow": 1.0e9,
        })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res.into_body()).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .starts_with("grow out of range"),
        "{body}"
    );

    let res = app
        .oneshot(rebake_request(serde_json::json!({
            "asset_id": Uuid::new_v4(),
            "prob_raster_id": "missing",
            "range": { "min": -1.0, "max": 2.0 },
        })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res.into_body()).await;
    assert_eq!(body["message"], "range must be within 0..1");
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn mask_generation_is_rejected_when_segmentation_is_off() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = Uuid::new_v4();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{id}/masks/generate"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "kind": "sky" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body_json(res.into_body()).await;
    assert_eq!(body["message"], "segmentation is disabled on this server");
}
