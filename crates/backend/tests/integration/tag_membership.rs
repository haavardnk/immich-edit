use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn other_asset() -> Uuid {
    Uuid::parse_str("bbbbbbbb-cccc-dddd-eeee-ffffffffffff").unwrap()
}

async fn mock_tagged(server: &MockServer, asset: Uuid, tag: &str, times: u64) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{asset}")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": asset,
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "tags": [{ "id": tag_id(), "name": tag, "value": tag }]
        })))
        .expect(times)
        .mount(server)
        .await;
}

async fn memberships(app: &axum::Router, ids: Value) -> axum::response::Response {
    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/tags/membership",
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
async fn tag_membership_answers_per_source_asset() {
    let server = MockServer::start().await;
    mock_tagged(&server, asset_id(), "Travel", 1).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}", other_asset())))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "message": "Not found or no asset.read access"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let asset = asset_id();

    let body = memberships_ok(&app, json!([asset, format!("{asset}_1"), other_asset()])).await;
    assert_eq!(body.as_object().unwrap().len(), 2, "{body}");
    assert_eq!(body[asset.to_string()][0]["value"], "Travel");
    assert_eq!(body[other_asset().to_string()], json!([]));
}

#[tokio::test]
async fn tag_membership_is_cached_until_a_tag_changes() {
    let server = MockServer::start().await;
    mock_tagged(&server, asset_id(), "Travel", 3).await;
    mock_tag_asset(&server).await;
    mock_tag_tree(&server).await;
    mock_server_version(&server, 3, 3, 0).await;
    Mock::given(method("PUT"))
        .and(path(format!("/api/tags/{}", tag_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": tag_id(),
            "name": "Trips",
            "value": "Trips"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    memberships_ok(&app, json!([asset_id()])).await;
    memberships_ok(&app, json!([asset_id()])).await;
    let tagged = app
        .clone()
        .oneshot(empty_request(
            "PUT",
            &format!("/api/tags/{}/assets/{}", tag_id(), asset_id()),
        ))
        .await
        .unwrap();
    assert_eq!(tagged.status(), StatusCode::OK);
    memberships_ok(&app, json!([asset_id()])).await;
    let renamed = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/tags/{}", tag_id()),
            json!({ "name": "Trips" }),
        ))
        .await
        .unwrap();
    assert_eq!(renamed.status(), StatusCode::OK);
    memberships_ok(&app, json!([asset_id()])).await;
}

#[tokio::test]
async fn tag_membership_bounds_the_batch() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let empty = memberships(&app, json!([])).await;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);

    let over: Vec<String> = (0..101u128)
        .map(|n| Uuid::from_u128(n + 1).to_string())
        .collect();
    let resp = memberships(&app, json!(over)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(resp).await["message"], "at most 100 assets");
}

#[tokio::test]
async fn tag_membership_surfaces_an_immich_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = memberships(&app, json!([asset_id()])).await;
    assert!(resp.status().is_server_error(), "{}", resp.status());
}

#[tokio::test]
async fn tag_membership_answers_the_rest_when_one_lookup_fails() {
    let server = MockServer::start().await;
    mock_tagged(&server, asset_id(), "Travel", 1).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}", other_asset())))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;

    let body = memberships_ok(&app, json!([asset_id(), other_asset()])).await;
    assert_eq!(body.as_object().unwrap().len(), 1, "{body}");
    assert_eq!(body[asset_id().to_string()][0]["value"], "Travel");
}
