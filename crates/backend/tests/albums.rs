mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::{self, header, method, path};
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
        .and(matchers::body_json(json!({ "ids": [asset] })))
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
        let body = body_json(resp).await;
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

#[tokio::test]
async fn lists_albums() {
    let server = MockServer::start().await;
    mock_albums(&server).await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/albums")).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["albumName"] != "Test Album" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn album_detail_returns_metadata_without_assets() {
    let server = MockServer::start().await;
    mock_album_detail(&server).await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/albums/{}", album_id())))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json["albumName"] != "Test Album" || json["assetCount"] != 1 {
        panic!("album: {json}");
    }
    if json.get("assets").is_some() {
        panic!("album still carries assets: {json}");
    }
}
