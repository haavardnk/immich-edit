use crate::common::*;
use axum::http::StatusCode;
use serde_json::json;
use wiremock::matchers::{body_json as body_is, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn capture_time_shifts_group_by_minutes_and_zone() {
    let server = MockServer::start().await;
    let (a, b, c) = (
        uuid::Uuid::from_u128(0xd1),
        uuid::Uuid::from_u128(0xd2),
        uuid::Uuid::from_u128(0xd3),
    );
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({ "ids": [a, b], "dateTimeRelative": -62 })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({
            "ids": [c],
            "dateTimeRelative": -122,
            "timeZone": "Europe/Oslo"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": a, "minutes": -62 },
        { "id": b, "minutes": -62 },
        { "id": c, "minutes": -122, "timeZone": "Europe/Oslo" },
        { "id": uuid::Uuid::from_u128(0xd4), "minutes": 0 }
    ] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/capture-time", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json, json!({ "failed": [] }));
}

#[tokio::test]
async fn capture_time_shifts_are_checked_before_immich() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let asset = asset_id();
    for body in [
        json!({ "items": [] }),
        json!({ "items": [{ "id": asset, "minutes": 2881 }] }),
        json!({ "items": [{ "id": asset, "minutes": 5, "timeZone": "Europe/Oslo; drop" }] }),
        json!({ "items": [{ "id": asset, "minutes": 1.5 }] }),
        json!({ "items": [{ "id": asset, "minutes": 5, "zone": "UTC" }] }),
    ] {
        let (status, _) =
            send_json(&app, "PUT", "/api/assets/capture-time", Some(body.clone())).await;
        if !status.is_client_error() {
            panic!("{body}: {status}");
        }
    }
}

#[tokio::test]
async fn capture_time_reports_the_groups_immich_refused() {
    let server = MockServer::start().await;
    let (a, b) = (uuid::Uuid::from_u128(0xe1), uuid::Uuid::from_u128(0xe2));
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({ "ids": [a], "dateTimeRelative": 5 })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({ "ids": [b], "dateTimeRelative": 7 })))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "message": "no" })))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [{ "id": a, "minutes": 5 }, { "id": b, "minutes": 7 }] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/capture-time", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json, json!({ "failed": [b] }));

    let only = json!({ "items": [{ "id": b, "minutes": 7 }] });
    let (status, _) = send_json(&app, "PUT", "/api/assets/capture-time", Some(only)).await;
    assert!(!status.is_success(), "{status}");
}
