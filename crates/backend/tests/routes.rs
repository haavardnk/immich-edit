mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::MockServer;

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

#[tokio::test]
async fn health_returns_ok_with_redacted_config() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let state = test_state(&server).await;
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let bytes = body_bytes(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let s = json.to_string();
    if s.contains("test-key") {
        panic!("api key leaked: {s}");
    }
    if json["renderer_mode"] != "cpu" {
        panic!("renderer_mode field");
    }
    if json["renderer_active"] != "cpu" {
        panic!("renderer_active field");
    }
    if json["immich_reachable"] != true {
        panic!("ping flag");
    }
    if json["immich_status"]["kind"] != "ok" {
        panic!("immich status: {}", json["immich_status"]);
    }
    if json["config"]["immich_api_key_present"] != true {
        panic!("redacted flag");
    }
}

#[tokio::test]
async fn health_reports_specific_immich_failure() {
    for (status, kind, code) in [
        (401u16, "api_key_rejected", serde_json::Value::Null),
        (503u16, "upstream_5xx", serde_json::json!(503)),
    ] {
        let server = MockServer::start().await;
        mock_ping_status(&server, status).await;
        let app = router(test_state(&server).await);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if resp.status() != StatusCode::OK {
            panic!("status {} for upstream {status}", resp.status());
        }
        let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
        if json["immich_reachable"] != false {
            panic!("ping flag for upstream {status}: {json}");
        }
        if json["immich_status"]["kind"] != kind {
            panic!("kind for upstream {status}: {json}");
        }
        if json["immich_status"]["status_code"] != code {
            panic!("status_code for upstream {status}: {json}");
        }
    }
}

#[tokio::test]
async fn lists_albums() {
    let server = MockServer::start().await;
    mock_albums(&server).await;
    let app = router(test_state(&server).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/albums")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["albumName"] != "Test Album" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn album_detail_returns_assets() {
    let server = MockServer::start().await;
    mock_album_detail(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/albums/{}", album_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["assets"][0]["originalFileName"] != "DSC0001.ARW" {
        panic!("asset: {json}");
    }
}

#[tokio::test]
async fn asset_thumb_proxies_bytes_and_content_type() {
    let server = MockServer::start().await;
    mock_thumb(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/thumb?size=preview", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !ct.starts_with("image/jpeg") {
        panic!("content-type: {ct}");
    }
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg soi");
    }
}

#[tokio::test]
async fn asset_thumb_rejects_bad_size() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}/thumb?size=nope", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::BAD_REQUEST {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn unknown_api_returns_json_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/does/not/exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !ct.contains("application/json") {
        panic!("content-type: {ct}");
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["code"] != "not_found" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn unknown_non_api_returns_plain_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/something")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if ct.contains("application/json") {
        panic!("non-api should not be JSON 404: {ct}");
    }
}

#[tokio::test]
async fn upstream_404_maps_to_404() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("status {}", resp.status());
    }
}

#[tokio::test]
async fn asset_detail_returns_exif_and_favorite() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{}", asset_id()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["isFavorite"] != true {
        panic!("favorite: {json}");
    }
    if json["exifInfo"]["rating"] != 4 {
        panic!("rating: {json}");
    }
    if json["exifInfo"]["exifImageWidth"] != 4032 {
        panic!("width: {json}");
    }
    if json["tags"][0]["value"] != "Landscape" {
        panic!("tags: {json}");
    }
}

#[tokio::test]
async fn asset_update_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_asset_update(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/assets/{}", asset_id()))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"rating":5,"isFavorite":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json["exifInfo"]["rating"] != 5 {
        panic!("rating: {json}");
    }
}

#[tokio::test]
async fn tags_upsert_proxies_to_immich() {
    let server = MockServer::start().await;
    mock_tag_upsert(&server).await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/tags")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"tags":["New"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["value"] != "New" {
        panic!("body: {json}");
    }
}

#[tokio::test]
async fn tag_asset_add_and_remove_proxy() {
    let server = MockServer::start().await;
    mock_tag_asset(&server).await;
    mock_untag_asset(&server).await;
    let app = router(test_state(&server).await);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/tags/{}/assets/{}", tag_id(), asset_id()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("put status {}", resp.status());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    if json[0]["success"] != true {
        panic!("body: {json}");
    }

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/tags/{}/assets/{}", tag_id(), asset_id()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("delete status {}", resp.status());
    }
}

#[tokio::test]
async fn raster_upload_get_roundtrip() {
    let server = MockServer::start().await;
    let state = test_state(&server).await;
    let app = router(state);
    let bytes = vec![0xABu8; 4 * 3];
    let resp = app
        .clone()
        .oneshot(
            Request::post("/api/rasters?width=4&height=3")
                .header("content-type", "application/octet-stream")
                .body(Body::from(bytes.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    let raster_id = v["raster_id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(
            Request::get(format!("/api/rasters/{raster_id}"))
                .body(Body::empty())
                .unwrap(),
        )
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
    let state = test_state(&server).await;
    let app = router(state);
    let resp = app
        .oneshot(
            Request::post("/api/rasters?width=4&height=3")
                .header("content-type", "application/octet-stream")
                .body(Body::from(vec![0u8; 5]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn live_endpoint_works_without_auth() {
    let server = MockServer::start().await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.auth_token = Some("secret".into());
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn protected_route_requires_auth_when_token_set() {
    let server = MockServer::start().await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.auth_token = Some("secret".into());
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_accepts_bearer_token() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.auth_token = Some("secret".into());
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn login_sets_cookie_then_protects() {
    let server = MockServer::start().await;
    mock_ping_ok(&server).await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.auth_token = Some("secret".into());
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"token":"secret"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let cookie = resp
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("immich_edit_auth=secret"));

    let send_cookie = cookie.split(';').next().unwrap().to_string();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("cookie", send_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn debug_timings_hidden_when_disabled() {
    let server = MockServer::start().await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.debug_endpoints = false;
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/debug/timings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn request_id_header_propagated() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.headers().get("x-request-id").is_some());
}

#[tokio::test]
async fn protected_route_rejects_invalid_token() {
    let server = MockServer::start().await;
    let mut state = test_state(&server).await;
    let mut cfg = (*state.config).clone();
    cfg.auth_token = Some("secret".into());
    state.config = std::sync::Arc::new(cfg);
    let app = router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("authorization", "Bearer wrong")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn error_body_request_id_matches_inbound_header() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/does/not/exist")
                .header("x-request-id", "client-supplied-1234")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        resp.headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap(),
        "client-supplied-1234"
    );
    let json: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    assert_eq!(json["request_id"], "client-supplied-1234");
}

#[tokio::test]
async fn create_job_expands_search_target_via_paging() {
    use uuid::Uuid;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, ResponseTemplate};

    let server = MockServer::start().await;
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(body_partial_json(serde_json::json!({ "page": "p2" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": { "items": [{ "id": id3, "type": "IMAGE" }], "count": 1, "total": 3, "nextPage": null }
        })))
        .with_priority(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": {
                "items": [{ "id": id1, "type": "IMAGE" }, { "id": id2, "type": "IMAGE" }],
                "count": 2, "total": 3, "nextPage": "p2"
            }
        })))
        .with_priority(5)
        .mount(&server)
        .await;

    let app = router(test_state(&server).await);
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "kind": "apply_preset",
                        "target": { "search": { "albumIds": [album_id()] } },
                        "params": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let job: serde_json::Value = serde_json::from_slice(&body_bytes(create).await).unwrap();
    assert_eq!(job["total"], 3);
    let job_id = job["id"].as_str().unwrap().to_string();

    let detail = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail: serde_json::Value = serde_json::from_slice(&body_bytes(detail).await).unwrap();
    let items = detail["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    let ids: Vec<&str> = items
        .iter()
        .map(|i| i["asset_id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&id3.to_string().as_str()));
}

#[tokio::test]
async fn create_job_without_ids_or_target_is_bad_request() {
    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "kind": "apply_preset", "params": {} }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_job_with_unknown_kind_is_bad_request() {
    use uuid::Uuid;

    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "kind": "not_a_real_kind",
                        "asset_ids": [Uuid::new_v4()],
                        "params": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = String::from_utf8(body_bytes(resp).await.to_vec()).unwrap();
    assert!(body.contains("unknown job kind"));
}

#[tokio::test]
async fn create_paste_edits_job_creates_items_for_ids() {
    use uuid::Uuid;

    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "kind": "paste_edits",
                        "asset_ids": [id1, id2],
                        "params": { "manifest": { "schema_version": 3, "ops": {} } }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let job: serde_json::Value = serde_json::from_slice(&body_bytes(create).await).unwrap();
    assert_eq!(job["kind"], "paste_edits");
    assert_eq!(job["total"], 2);
}

#[tokio::test]
async fn create_reset_edits_job_creates_items_for_ids() {
    use uuid::Uuid;

    let server = MockServer::start().await;
    let app = router(test_state(&server).await);
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "kind": "reset_edits",
                        "asset_ids": [id1, id2],
                        "params": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let job: serde_json::Value = serde_json::from_slice(&body_bytes(create).await).unwrap();
    assert_eq!(job["kind"], "reset_edits");
    assert_eq!(job["total"], 2);
}
