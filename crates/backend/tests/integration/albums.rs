use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
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
async fn lists_the_albums_holding_an_asset_by_its_source() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(matchers::query_param("assetId", asset_id().to_string()))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "id": album_id(),
            "albumName": "Bergen 2026",
            "assetCount": 148,
            "albumThumbnailAssetId": asset_id()
        }])))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/albums?assetId={}_2", asset_id())))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json = body_json(resp).await;
    if json[0]["albumName"] != "Bergen 2026" || json[0]["assetCount"] != 148 {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn album_list_rejects_a_malformed_asset_id() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get("/api/albums?assetId=not-an-id"))
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
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

#[tokio::test]
async fn album_list_carries_owner_sharing_and_newest_photo() {
    let server = MockServer::start().await;
    let member = |id, role| json!({ "user": { "id": id, "name": "", "email": "" }, "role": role });
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "id": album_id(),
            "albumName": "Family",
            "assetCount": 1024,
            "shared": true,
            "hasSharedLink": false,
            "endDate": "2026-07-14T10:00:00.000Z",
            "albumUsers": [
                member(member_user_id(), "owner"),
                member(test_user_id(), "editor")
            ]
        }])))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/albums")).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let album = &body_json(resp).await[0];
    if album["ownerId"] != json!(member_user_id())
        || album["shared"] != true
        || album["endDate"] != "2026-07-14T10:00:00.000Z"
    {
        panic!("album: {album}");
    }
}

#[tokio::test]
async fn album_list_leaves_owner_and_newest_photo_unknown_without_them() {
    let server = MockServer::start().await;
    mock_albums(&server).await;
    let app = test_app(&server).await;
    let resp = app.oneshot(get("/api/albums")).await.unwrap();
    let album = &body_json(resp).await[0];
    if !album["ownerId"].is_null() || album["shared"] != false || !album["endDate"].is_null() {
        panic!("album: {album}");
    }
}

#[tokio::test]
async fn creating_an_album_sends_the_trimmed_name_with_the_session_credential() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/albums"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(matchers::body_json(json!({ "albumName": "Norway 2026" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": album_id(),
            "albumName": "Norway 2026",
            "assetCount": 0,
            "albumThumbnailAssetId": null,
            "shared": false,
            "hasSharedLink": false,
            "updatedAt": "2026-11-08T09:00:00.000Z",
            "albumUsers": [{ "user": { "id": test_user_id() }, "role": "owner" }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/albums",
            json!({ "albumName": "  Norway 2026 " }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let album = body_json(resp).await;
    if album["id"] != json!(album_id())
        || album["albumName"] != "Norway 2026"
        || album["ownerId"] != json!(test_user_id())
        || album["assetCount"] != 0
    {
        panic!("album: {album}");
    }
}

#[tokio::test]
async fn creating_an_album_rejects_a_blank_long_or_control_name_before_immich() {
    for name in [
        "   ".to_string(),
        "x".repeat(81),
        "Trip\nNorway".to_string(),
        "Trip\u{7}".to_string(),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/albums"))
            .respond_with(ResponseTemplate::new(201))
            .expect(0)
            .mount(&server)
            .await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(json_request(
                "POST",
                "/api/albums",
                json!({ "albumName": name }),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::BAD_REQUEST {
            panic!("name {name:?} returned {}", resp.status());
        }
    }
}

#[tokio::test]
async fn creating_an_album_surfaces_an_immich_refusal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/albums"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({ "message": "Forbidden" })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/albums",
            json!({ "albumName": "Norway" }),
        ))
        .await
        .unwrap();
    if resp.status().is_success() {
        panic!("refused create returned {}", resp.status());
    }
}

#[tokio::test]
async fn creating_an_album_sends_a_trimmed_description_and_omits_a_blank_one() {
    for (sent, forwarded) in [
        (
            json!("  Fjords\nand rain "),
            json!({ "albumName": "Norway", "description": "Fjords\nand rain" }),
        ),
        (json!("   "), json!({ "albumName": "Norway" })),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/albums"))
            .and(matchers::body_json(forwarded.clone()))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": album_id(),
                "albumName": "Norway",
                "description": forwarded.get("description").cloned().unwrap_or(json!(""))
            })))
            .expect(1)
            .mount(&server)
            .await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(json_request(
                "POST",
                "/api/albums",
                json!({ "albumName": "Norway", "description": sent }),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("description {sent} returned {}", resp.status());
        }
    }
}

#[tokio::test]
async fn album_detail_carries_description_dates_and_member_roles() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": album_id(),
            "albumName": "Family",
            "description": "Summer at the lake",
            "assetCount": 12,
            "startDate": "2026-06-01T10:00:00.000Z",
            "endDate": "2026-07-14T10:00:00.000Z",
            "shared": true,
            "albumUsers": [
                { "user": { "id": member_user_id(), "name": "Ann", "email": "a@x" }, "role": "owner" },
                { "user": { "id": test_user_id(), "name": "Bo", "email": "b@x" }, "role": "editor" }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/albums/{}", album_id())))
        .await
        .unwrap();
    let album = body_json(resp).await;
    let members = json!([
        { "userId": member_user_id(), "name": "Ann", "role": "owner" },
        { "userId": test_user_id(), "name": "Bo", "role": "editor" }
    ]);
    if album["description"] != "Summer at the lake"
        || album["startDate"] != "2026-06-01T10:00:00.000Z"
        || album["endDate"] != "2026-07-14T10:00:00.000Z"
        || album["ownerId"] != json!(member_user_id())
        || album["members"] != members
    {
        panic!("album: {album}");
    }
}

#[tokio::test]
async fn album_detail_defaults_description_and_members_when_absent() {
    let server = MockServer::start().await;
    mock_album_detail(&server).await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(get(&format!("/api/albums/{}", album_id())))
        .await
        .unwrap();
    let album = body_json(resp).await;
    if album["description"] != "" || album["members"] != json!([]) {
        panic!("album: {album}");
    }
}

async fn mock_album_patch(server: &MockServer, forwarded: Value) {
    Mock::given(method("PATCH"))
        .and(path(format!("/api/albums/{}", album_id())))
        .and(header("x-api-key", TEST_API_KEY))
        .and(matchers::body_json(forwarded))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": album_id(),
            "albumName": "stale",
            "assetCount": 0
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn updating_an_album_patches_immich_and_returns_the_refetched_album() {
    let server = MockServer::start().await;
    mock_album_patch(
        &server,
        json!({
            "albumName": "Norway",
            "description": "",
            "albumThumbnailAssetId": asset_id()
        }),
    )
    .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": album_id(),
            "albumName": "Norway",
            "assetCount": 148,
            "albumThumbnailAssetId": asset_id(),
            "albumUsers": [{ "user": { "id": test_user_id() }, "role": "owner" }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/albums/{}", album_id()),
            json!({
                "albumName": " Norway ",
                "description": "  ",
                "albumThumbnailAssetId": format!("{}_1", asset_id())
            }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let album = body_json(resp).await;
    if album["albumName"] != "Norway"
        || album["assetCount"] != 148
        || album["members"][0]["role"] != "owner"
    {
        panic!("album: {album}");
    }
}

#[tokio::test]
async fn updating_an_album_forwards_only_the_fields_sent() {
    let server = MockServer::start().await;
    mock_album_patch(&server, json!({ "description": "Day one" })).await;
    mock_album_detail(&server).await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/albums/{}", album_id()),
            json!({ "description": "Day one " }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn updating_an_album_rejects_empty_unknown_or_invalid_fields_before_immich() {
    for body in [
        json!({}),
        json!({ "order": "asc" }),
        json!({ "albumName": "  " }),
        json!({ "albumName": "x".repeat(81) }),
        json!({ "description": "x".repeat(2001) }),
        json!({ "description": "Day\u{7}" }),
        json!({ "albumThumbnailAssetId": "not-an-id" }),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let app = test_app(&server).await;
        let resp = app
            .oneshot(json_request(
                "PATCH",
                &format!("/api/albums/{}", album_id()),
                body.clone(),
            ))
            .await
            .unwrap();
        if !resp.status().is_client_error() {
            panic!("body {body} returned {}", resp.status());
        }
    }
}

#[tokio::test]
async fn an_immich_refusal_to_update_surfaces_without_a_refetch() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/albums/{}", album_id())))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "message": "Invalid album thumbnail" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/albums/{}", album_id()),
            json!({ "albumThumbnailAssetId": asset_id() }),
        ))
        .await
        .unwrap();
    if resp.status().is_success() {
        panic!("refused update returned {}", resp.status());
    }
}

async fn mock_album_delete(server: &MockServer, uri: String, key: &str, status: u16) {
    Mock::given(method("DELETE"))
        .and(path(uri))
        .and(header("x-api-key", key))
        .respond_with(ResponseTemplate::new(status))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn deleting_an_album_forwards_the_session_credential() {
    let server = MockServer::start().await;
    mock_album_delete(
        &server,
        format!("/api/albums/{}", album_id()),
        TEST_API_KEY,
        204,
    )
    .await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(empty_request(
            "DELETE",
            &format!("/api/albums/{}", album_id()),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn leaving_an_album_removes_the_session_user_by_id() {
    let server = MockServer::start().await;
    let uri = format!("/api/albums/{}/user/{}", album_id(), member_user_id());
    mock_album_delete(&server, uri, MEMBER_API_KEY, 200).await;
    let (_admin, app) = two_owner_apps(&server).await;
    let resp = app
        .oneshot(empty_request(
            "DELETE",
            &format!("/api/albums/{}/membership", album_id()),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn an_immich_refusal_to_delete_or_leave_surfaces() {
    for (route, upstream) in [
        (
            format!("/api/albums/{}", album_id()),
            format!("/api/albums/{}", album_id()),
        ),
        (
            format!("/api/albums/{}/membership", album_id()),
            format!("/api/albums/{}/user/{}", album_id(), test_user_id()),
        ),
    ] {
        let server = MockServer::start().await;
        mock_album_delete(&server, upstream, TEST_API_KEY, 400).await;
        let app = test_app(&server).await;
        let resp = app.oneshot(empty_request("DELETE", &route)).await.unwrap();
        if resp.status().is_success() {
            panic!("refused {route} returned {}", resp.status());
        }
    }
}
