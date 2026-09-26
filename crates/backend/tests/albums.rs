mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn album_assets_req(verb: &str, ids: Value) -> Request<Body> {
    Request::builder()
        .method(verb)
        .uri(format!("/api/albums/{}/assets", album_id()))
        .header("content-type", "application/json")
        .body(Body::from(json!({ "ids": ids }).to_string()))
        .unwrap()
}

async fn mock_album_assets(server: &MockServer, verb: &str, key: &str, status: u16) {
    let asset = asset_id();
    Mock::given(method(verb))
        .and(path(format!("/api/albums/{}/assets", album_id())))
        .and(header("x-api-key", key))
        .and(body_json(json!({ "ids": [asset] })))
        .respond_with(
            ResponseTemplate::new(status).set_body_json(json!([{ "id": asset, "success": true }])),
        )
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn album_membership_forwards_source_ids_once() {
    for verb in ["PUT", "DELETE"] {
        let server = MockServer::start().await;
        mock_album_assets(&server, verb, TEST_API_KEY, 200).await;
        let app = test_app(&server).await;
        let asset = asset_id();

        let resp = app
            .oneshot(album_assets_req(verb, json!([asset, format!("{asset}_1")])))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("{verb} album assets returned {}", resp.status());
        }
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        if body[0]["success"] != true {
            panic!("{verb} album assets body: {body}");
        }
    }
}

#[tokio::test]
async fn album_membership_rejects_an_empty_selection() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(album_assets_req("PUT", json!([])))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("empty album add returned {}", resp.status());
    }
}

#[tokio::test]
async fn another_users_album_is_refused() {
    let server = MockServer::start().await;
    mock_album_assets(&server, "PUT", MEMBER_API_KEY, 400).await;
    let (_admin, app) = two_owner_apps(&server).await;
    let resp = app
        .oneshot(album_assets_req("PUT", json!([asset_id()])))
        .await
        .unwrap();
    if resp.status().is_success() {
        panic!("member add to a foreign album returned {}", resp.status());
    }
}
