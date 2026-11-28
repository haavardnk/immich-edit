use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn other_asset() -> Uuid {
    Uuid::parse_str("bbbbbbbb-cccc-dddd-eeee-ffffffffffff").unwrap()
}

async fn mock_albums_with(server: &MockServer, asset: Uuid, name: &str, times: u64) {
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(query_param("assetId", asset.to_string()))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "id": album_id(),
            "albumName": name,
            "assetCount": 3
        }])))
        .expect(times)
        .mount(server)
        .await;
}

async fn memberships(app: &axum::Router, ids: Value) -> axum::response::Response {
    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/albums/membership",
            json!({ "ids": ids }),
        ))
        .await
        .unwrap()
}

async fn memberships_ok(app: &axum::Router, ids: Value) -> Value {
    let resp = memberships(app, ids.clone()).await;
    assert_eq!(resp.status(), StatusCode::OK, "{ids}");
    body_json(resp).await
}

#[tokio::test]
async fn album_membership_answers_per_source_asset() {
    let server = MockServer::start().await;
    mock_albums_with(&server, asset_id(), "Bergen", 1).await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(query_param("assetId", other_asset().to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let asset = asset_id();

    let body = memberships_ok(&app, json!([asset, format!("{asset}_1"), other_asset()])).await;
    let map = body.as_object().unwrap();
    assert_eq!(map.len(), 2, "{body}");
    assert_eq!(body[asset.to_string()][0]["albumName"], "Bergen");
    assert_eq!(body[other_asset().to_string()], json!([]));
}

#[tokio::test]
async fn album_membership_is_cached_until_membership_changes() {
    let server = MockServer::start().await;
    mock_albums_with(&server, asset_id(), "Bergen", 2).await;
    Mock::given(method("PUT"))
        .and(path(format!("/api/albums/{}/assets", album_id())))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([{ "id": asset_id(), "success": true }])),
        )
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    memberships_ok(&app, json!([asset_id()])).await;
    memberships_ok(&app, json!([asset_id()])).await;
    let added = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/albums/{}/assets", album_id()),
            json!({ "ids": [asset_id()] }),
        ))
        .await
        .unwrap();
    assert_eq!(added.status(), StatusCode::OK);
    memberships_ok(&app, json!([asset_id()])).await;
}

#[tokio::test]
async fn album_membership_refetches_after_an_album_rename() {
    let server = MockServer::start().await;
    mock_albums_with(&server, asset_id(), "Bergen", 2).await;
    let renamed = json!({ "id": album_id(), "albumName": "Oslo", "assetCount": 3 });
    Mock::given(method("PATCH"))
        .and(path(format!("/api/albums/{}", album_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(renamed.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(renamed))
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    memberships_ok(&app, json!([asset_id()])).await;
    let patched = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/albums/{}", album_id()),
            json!({ "albumName": "Oslo" }),
        ))
        .await
        .unwrap();
    assert_eq!(patched.status(), StatusCode::OK);
    memberships_ok(&app, json!([asset_id()])).await;
}

#[tokio::test]
async fn album_membership_bounds_the_batch() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let asset = asset_id();

    let empty = memberships(&app, json!([])).await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    let over: Vec<String> = (0..101u128)
        .map(|n| Uuid::from_u128(n + 1).to_string())
        .collect();
    let resp = memberships(&app, json!(over)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let error = body_json(resp).await;
    assert_eq!(error["message"], "at most 100 assets");

    let copies: Vec<String> = (1..=150).map(|n| format!("{asset}_{n}")).collect();
    mock_albums_with(&server, asset, "Bergen", 1).await;
    let body = memberships_ok(&app, json!(copies)).await;
    assert_eq!(body.as_object().unwrap().len(), 1, "{body}");
}

#[tokio::test]
async fn album_membership_refuses_bad_shapes() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for body in [
        json!({ "ids": ["not-an-id"] }),
        json!({ "ids": [asset_id()], "limit": 5 }),
        json!({}),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/albums/membership", body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    }
}

#[tokio::test]
async fn album_membership_surfaces_an_immich_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = memberships(&app, json!([asset_id()])).await;
    assert!(resp.status().is_server_error(), "{}", resp.status());
}

#[tokio::test]
async fn album_membership_answers_the_rest_when_one_lookup_fails() {
    let server = MockServer::start().await;
    mock_albums_with(&server, asset_id(), "Bergen", 1).await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(query_param("assetId", other_asset().to_string()))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let body = memberships_ok(&app, json!([asset_id(), other_asset()])).await;
    assert_eq!(body.as_object().unwrap().len(), 1, "{body}");
    assert_eq!(body[asset_id().to_string()][0]["albumName"], "Bergen");
}

#[tokio::test]
async fn album_membership_answers_no_albums_for_an_unreadable_asset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let body = memberships_ok(&app, json!([asset_id()])).await;
    assert_eq!(body[asset_id().to_string()], json!([]));
}
