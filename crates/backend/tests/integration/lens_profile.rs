use crate::common::*;
use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_asset_named(server: &MockServer, id: Uuid, file_name: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "originalFileName": file_name,
            "type": "IMAGE",
            "ownerId": test_user_id(),
            "exifInfo": { "make": "SONY", "model": "ILCE-7M3", "lensModel": "FE 35mm F1.8" },
        })))
        .mount(server)
        .await;
}

async fn lens_auto(app: &axum::Router, id: Uuid) -> serde_json::Value {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/assets/{id}/lens-profile"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(res).await["auto"].clone()
}

#[tokio::test]
async fn embedded_dng_corrections_turn_the_automatic_lens_profile_off() {
    let server = MockServer::start().await;
    let cases = [
        ("DJI_0765.DNG", "DJI_FC7303_16-9.dng", false),
        ("R0000001.dng", "Ricoh_GR_12bit_4-3.dng", true),
    ];
    let mut ids = Vec::new();
    for (name, fixture, _) in cases {
        let id = Uuid::new_v4();
        mock_asset_named(&server, id, name).await;
        mock_original_with(&server, id, raw_response(fixture, "image/x-adobe-dng")).await;
        ids.push(id);
    }
    let arw = Uuid::new_v4();
    mock_asset_named(&server, arw, "DSC0001.ARW").await;
    let app = test_app(&server).await;

    for ((name, _, expected), id) in cases.iter().zip(&ids) {
        assert_eq!(lens_auto(&app, *id).await, *expected, "{name}");
    }
    assert_eq!(lens_auto(&app, arw).await, true);
    assert_eq!(originals_fetched(&server, arw).await, 0);
}
