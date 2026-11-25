use crate::common::search_library::{Photo, mount};
use crate::common::*;
use axum::http::StatusCode;
use chrono::{DateTime, TimeDelta, Utc};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

const STRUCTURED: u64 = 3;
const FLAT: u64 = 1;

fn shot(taken: &str, zone_hours: i64) -> Photo {
    Photo {
        id: Uuid::new_v4(),
        taken: DateTime::parse_from_rfc3339(taken)
            .unwrap()
            .with_timezone(&Utc),
        zone: TimeDelta::hours(zone_hours),
        ..Photo::default()
    }
}

async fn library(minor: u64, photos: &[Photo]) -> (MockServer, axum::Router) {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, minor, 2).await;
    mount(&server, TEST_API_KEY, photos.to_vec()).await;
    let app = test_app(&server).await;
    (server, app)
}

async fn post(app: &axum::Router, uri: &str, body: Value) -> Value {
    let resp = app
        .clone()
        .oneshot(json_request("POST", uri, body.clone()))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("{uri} {body} gave {}", resp.status());
    }
    body_json(resp).await
}

fn ids(items: &Value) -> Vec<Uuid> {
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().parse().unwrap())
        .collect()
}

struct Trip {
    camera_noon: Photo,
    phone_half_past: Photo,
    east_after_midnight: Photo,
}

fn trip() -> Trip {
    Trip {
        camera_noon: shot("2025-03-31T12:00:00Z", 0),
        phone_half_past: shot("2025-03-31T10:30:00Z", 2),
        east_after_midnight: shot("2025-03-31T22:30:00Z", 3),
    }
}

impl Trip {
    fn photos(&self) -> [Photo; 3] {
        [
            self.camera_noon,
            self.phone_half_past,
            self.east_after_midnight,
        ]
    }
}

#[tokio::test]
async fn taken_pages_follow_each_photos_clock_on_structured_servers() {
    let trip = trip();
    let by_photo_clock = vec![
        trip.camera_noon.id,
        trip.phone_half_past.id,
        trip.east_after_midnight.id,
    ];
    let by_instant = vec![
        trip.phone_half_past.id,
        trip.camera_noon.id,
        trip.east_after_midnight.id,
    ];
    for (minor, expected) in [(STRUCTURED, by_photo_clock), (FLAT, by_instant)] {
        let (_server, app) = library(minor, &trip.photos()).await;
        let page = post(
            &app,
            "/api/search/metadata",
            json!({ "sort": { "field": "taken", "direction": "asc" }, "size": 10 }),
        )
        .await;
        if ids(&page["items"]) != expected {
            panic!("3.{minor} listed {page}");
        }
    }
}

#[tokio::test]
async fn months_locate_and_day_counts_bucket_by_each_photos_clock() {
    let trip = trip();
    for (minor, march, april) in [(STRUCTURED, 2, 1), (FLAT, 3, 0)] {
        let (_server, app) = library(minor, &trip.photos()).await;
        let months = post(&app, "/api/search/months", json!({ "filter": {} })).await;
        let mut expected = vec![json!({ "month": "2025-03", "count": march })];
        if april > 0 {
            expected.insert(0, json!({ "month": "2025-04", "count": april }));
        }
        if months != json!({ "months": expected }) {
            panic!("3.{minor} months {months}");
        }
        let days = post(
            &app,
            "/api/search/counts",
            json!({ "filter": {}, "edges": [
                "2025-03-31T00:00:00Z", "2025-04-01T00:00:00Z", "2025-04-02T00:00:00Z"
            ] }),
        )
        .await;
        if days != json!({ "counts": [march, april] }) {
            panic!("3.{minor} day counts {days}");
        }
    }
    let (_server, app) = library(STRUCTURED, &trip.photos()).await;
    let located = post(
        &app,
        "/api/search/locate",
        json!({ "filter": {}, "direction": "desc", "month": "2025-03", "offset": 0 }),
    )
    .await;
    if located != json!({ "assetId": trip.phone_half_past.id }) {
        panic!("locate gave {located}");
    }
}

#[tokio::test]
async fn a_window_walks_each_photos_clock_on_structured_servers() {
    let trip = trip();
    let (_server, app) = library(STRUCTURED, &trip.photos()).await;
    let window = post(
        &app,
        "/api/search/window",
        json!({
            "assetId": trip.phone_half_past.id,
            "filter": {},
            "sort": { "field": "taken", "direction": "asc" },
            "radius": 5
        }),
    )
    .await;
    let expected = vec![
        trip.camera_noon.id,
        trip.phone_half_past.id,
        trip.east_after_midnight.id,
    ];
    if ids(&window["items"]) != expected {
        panic!("window was {window}");
    }
}

fn noon_in(zone_hours: i64) -> Photo {
    let photo = shot("2025-06-01T12:00:00Z", zone_hours);
    Photo {
        taken: photo.taken - TimeDelta::hours(zone_hours),
        ..photo
    }
}

fn search_calls(requests: &[wiremock::Request]) -> usize {
    requests
        .iter()
        .filter(|request| request.url.path() == "/api/search/metadata")
        .count()
}

#[tokio::test]
async fn a_window_breaks_ties_by_id_and_needs_no_boundary_scans() {
    let mut tied: Vec<Photo> = [0, 2, -5, 3].into_iter().map(noon_in).collect();
    tied.sort_by_key(|photo| photo.id);
    let before = shot("2025-06-01T08:00:00Z", 0);
    let after = shot("2025-06-01T15:00:00Z", 0);
    let mut photos = tied.clone();
    photos.extend([before, after]);
    let (server, app) = library(STRUCTURED, &photos).await;
    let anchor = tied[1].id;
    let start = server.received_requests().await.unwrap_or_default().len();
    let window = post(
        &app,
        "/api/search/window",
        json!({
            "assetId": anchor,
            "filter": {},
            "sort": { "field": "taken", "direction": "asc" },
            "radius": 10
        }),
    )
    .await;
    let mut expected = vec![before.id];
    expected.extend(tied.iter().map(|photo| photo.id));
    expected.push(after.id);
    if ids(&window["items"]) != expected {
        panic!("window was {window}");
    }
    let requests = server.received_requests().await.unwrap_or_default();
    let calls = search_calls(&requests[start..]);
    if calls != 3 {
        panic!("the window made {calls} metadata calls");
    }
}

#[tokio::test]
async fn a_picked_range_clamps_the_photo_clock_months() {
    let trip = trip();
    let (_server, app) = library(STRUCTURED, &trip.photos()).await;
    let march =
        json!({ "takenLocal": { "from": "2025-03-01T00:00:00", "until": "2025-04-01T00:00:00" } });
    let months = post(&app, "/api/search/months", json!({ "filter": march })).await;
    if months != json!({ "months": [{ "month": "2025-03", "count": 2 }] }) {
        panic!("months were {months}");
    }
}
