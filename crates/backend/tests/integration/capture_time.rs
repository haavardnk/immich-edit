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
async fn capture_time_sets_times_to_the_second_with_their_zone() {
    let server = MockServer::start().await;
    let (a, b, c) = (
        uuid::Uuid::from_u128(0xf1),
        uuid::Uuid::from_u128(0xf2),
        uuid::Uuid::from_u128(0xf3),
    );
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({
            "ids": [a, b],
            "dateTimeOriginal": "1987-06-01T12:00:00.000+02:00",
            "timeZone": "Europe/Oslo"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({
            "ids": [c],
            "dateTimeOriginal": "2026-10-31T14:05:49.345+01:00",
            "timeZone": "UTC+1"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(
            json!({ "ids": [uuid::Uuid::from_u128(0xf4)], "dateTimeRelative": 15_778_800 }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": a, "at": "1987-06-01T12:00:00+02:00", "timeZone": "Europe/Oslo" },
        { "id": b, "at": "1987-06-01T10:00:00Z", "timeZone": "Europe/Oslo" },
        { "id": c, "at": "2026-10-31T14:05:49.345+01:00", "timeZone": "UTC+1" },
        { "id": uuid::Uuid::from_u128(0xf4), "minutes": 15_778_800 }
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
    let refused = [
        json!({ "items": [] }),
        json!({ "items": [{ "id": asset, "minutes": 105_408_001 }] }),
        json!({ "items": [{ "id": asset, "minutes": i64::MIN }] }),
        json!({ "items": [{ "id": asset }] }),
        json!({ "items": [{ "id": asset, "minutes": 5, "at": "2026-10-31T14:05:49+01:00", "timeZone": "UTC+1" }] }),
        json!({ "items": [{ "id": asset, "at": "2026-10-31T14:05:49+01:00" }] }),
        json!({ "items": [{ "id": asset, "at": "1800-12-31T23:59:59Z", "timeZone": "UTC" }] }),
        json!({ "items": [{ "id": asset, "at": "9999-01-01T00:00:00Z", "timeZone": "UTC" }] }),
        json!({ "items": [{ "id": asset, "minutes": 5, "timeZone": "Europe/Oslo; drop" }] }),
    ];
    for body in refused {
        let (status, _) =
            send_json(&app, "PUT", "/api/assets/capture-time", Some(body.clone())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
    let malformed = [
        json!({ "items": [{ "id": asset, "at": "2026-10-31T14:05:49", "timeZone": "UTC+1" }] }),
        json!({ "items": [{ "id": asset, "minutes": 1.5 }] }),
        json!({ "items": [{ "id": asset, "minutes": 5, "zone": "UTC" }] }),
    ];
    for body in malformed {
        let (status, _) =
            send_json(&app, "PUT", "/api/assets/capture-time", Some(body.clone())).await;
        assert!(status.is_client_error(), "{body}: {status}");
    }
}

#[tokio::test]
async fn capture_time_accepts_the_limits_and_splits_zones() {
    let server = MockServer::start().await;
    let ids = Vec::from_fn(6, |i| uuid::Uuid::from_u128(0xa0 + i as u128));
    for update in [
        json!({ "ids": [ids[0]], "dateTimeRelative": 105_408_000 }),
        json!({ "ids": [ids[1]], "dateTimeRelative": -105_408_000 }),
        json!({ "ids": [ids[2]], "dateTimeOriginal": "1801-01-01T00:00:00.000+00:00", "timeZone": "UTC" }),
        json!({ "ids": [ids[3]], "dateTimeOriginal": "9998-12-31T23:59:59.999+00:00", "timeZone": "UTC" }),
        json!({ "ids": [ids[4]], "dateTimeOriginal": "9998-12-31T23:59:59.999+00:00", "timeZone": "UTC+1" }),
        json!({ "ids": [ids[5]], "dateTimeRelative": 0, "timeZone": "Asia/Bangkok" }),
    ] {
        Mock::given(method("PUT"))
            .and(path("/api/assets"))
            .and(body_is(update))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
    }
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": ids[0], "minutes": 105_408_000 },
        { "id": ids[1], "minutes": -105_408_000 },
        { "id": ids[2], "at": "1801-01-01T00:00:00Z", "timeZone": "UTC" },
        { "id": ids[3], "at": "9998-12-31T23:59:59.9991Z", "timeZone": "UTC" },
        { "id": ids[4], "at": "9998-12-31T23:59:59.9992Z", "timeZone": "UTC+1" },
        { "id": ids[5], "minutes": 0, "timeZone": "Asia/Bangkok" }
    ] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/capture-time", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
}

#[tokio::test]
async fn capture_time_sends_copies_as_their_source_once() {
    let server = MockServer::start().await;
    let (a, b) = (uuid::Uuid::from_u128(0xb1), uuid::Uuid::from_u128(0xb2));
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({
            "ids": [a],
            "dateTimeOriginal": "2026-10-31T14:05:49.000+01:00",
            "timeZone": "UTC+1"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(json!({ "ids": [b], "dateTimeRelative": 3 })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": format!("{a}_1"), "at": "2026-10-31T14:05:49+01:00", "timeZone": "UTC+1" },
        { "id": a, "minutes": 62 },
        { "id": b, "minutes": 0 },
        { "id": format!("{b}_2"), "minutes": 3 }
    ] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/capture-time", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
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
