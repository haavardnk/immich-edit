use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_smart_proxies_to_immich() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/smart"))
        .and(body_partial_json(json!({ "query": "red car" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "assets": {
                "items": [{ "id": asset_id(), "type": "IMAGE" }],
                "count": 1, "total": 1, "nextPage": null
            }
        })))
        .mount(&server)
        .await;

    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/search/smart",
            json!({ "query": "red car" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["items"][0]["id"], asset_id().to_string());
    assert_eq!(json["total"], 1);
}
