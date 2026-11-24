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

async fn feed(app: &axum::Router, filter: &Value) -> Vec<Uuid> {
    let mut ids = Vec::new();
    let mut cursor = Value::Null;
    loop {
        let mut body = json!({ "filter": filter, "size": 1 });
        if !cursor.is_null() {
            body["cursor"] = cursor;
        }
        let page = post(app, "/api/search/metadata", body).await;
        ids.extend(
            page["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["id"].as_str().unwrap().parse::<Uuid>().unwrap()),
        );
        cursor = page["nextCursor"].clone();
        if cursor.is_null() {
            return ids;
        }
    }
}

fn picked_day() -> Value {
    json!({ "takenLocal": { "from": "2025-03-18T00:00:00", "until": "2025-03-19T00:00:00" } })
}

#[tokio::test]
async fn a_picked_day_holds_the_photos_whose_own_clock_reads_that_day() {
    let early_east = shot("2025-03-17T22:00:00Z", 3);
    let west_evening_before = shot("2025-03-18T03:00:00Z", -5);
    let morning_east = shot("2025-03-18T05:00:00Z", 3);
    let past_midnight_east = shot("2025-03-18T21:30:00Z", 3);
    let evening_west = shot("2025-03-18T23:30:00Z", -5);
    let day_before = shot("2025-03-17T12:00:00Z", 0);
    let photos = [
        early_east,
        west_evening_before,
        morning_east,
        past_midnight_east,
        evening_west,
        day_before,
    ];
    let expected = vec![evening_west.id, morning_east.id, early_east.id];
    for minor in [FLAT, STRUCTURED] {
        let (_server, app) = library(minor, &photos).await;
        let found = feed(&app, &picked_day()).await;
        if found != expected {
            panic!("3.{minor} listed {found:?}, expected {expected:?}");
        }
        let total = post(
            &app,
            "/api/search/statistics",
            json!({ "filter": picked_day() }),
        )
        .await;
        if total != json!({ "total": 3 }) {
            panic!("3.{minor} counted {total}");
        }
        let months = post(
            &app,
            "/api/search/months",
            json!({ "filter": picked_day() }),
        )
        .await;
        if months != json!({ "months": [{ "month": "2025-03", "count": 3 }] }) {
            panic!("3.{minor} counted months {months}");
        }
    }
}

#[tokio::test]
async fn a_day_of_one_zone_sends_tight_bounds_without_extra_pages() {
    let morning = shot("2025-03-18T05:00:00Z", 3);
    let next_night = shot("2025-03-18T21:30:00Z", 3);
    let (server, app) = library(STRUCTURED, &[morning, next_night]).await;
    let page = post(
        &app,
        "/api/search/metadata",
        json!({ "filter": picked_day(), "size": 10 }),
    )
    .await;
    if page["items"].as_array().map(Vec::len) != Some(1) {
        panic!("one-zone day listed {page}");
    }
    let requests = server.received_requests().await.unwrap_or_default();
    let last = requests
        .iter()
        .rev()
        .find(|request| request.url.path() == "/api/search/metadata")
        .and_then(|request| request.body_json::<Value>().ok())
        .unwrap();
    let bounds = &last["filter"]["takenAt"];
    if *bounds != json!({ "gte": "2025-03-18T05:00:00.000Z", "lt": "2025-03-18T05:00:00.001Z" }) {
        panic!("the page was bounded by {bounds}");
    }
}

#[tokio::test]
async fn an_empty_day_never_lists_or_counts_its_neighbours() {
    let next_night = shot("2025-03-18T21:30:00Z", 3);
    for minor in [FLAT, STRUCTURED] {
        let (_server, app) = library(minor, &[next_night]).await;
        if !feed(&app, &picked_day()).await.is_empty() {
            panic!("3.{minor} listed a photo of the next day");
        }
        let total = post(
            &app,
            "/api/search/statistics",
            json!({ "filter": picked_day() }),
        )
        .await;
        if total != json!({ "total": 0 }) {
            panic!("3.{minor} counted {total}");
        }
    }
}

fn ids(items: &Value) -> Vec<Uuid> {
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().parse().unwrap())
        .collect()
}

#[tokio::test]
async fn a_window_never_anchors_on_or_reaches_a_photo_of_another_day() {
    let early_east = shot("2025-03-17T22:00:00Z", 3);
    let west_evening_before = shot("2025-03-18T03:00:00Z", -5);
    let morning_east = shot("2025-03-18T05:00:00Z", 3);
    let photos = [early_east, west_evening_before, morning_east];
    for minor in [FLAT, STRUCTURED] {
        let (_server, app) = library(minor, &photos).await;
        let window = |anchor: Uuid| {
            json!({
                "assetId": anchor,
                "filter": picked_day(),
                "sort": { "field": "taken", "direction": "desc" },
                "radius": 5
            })
        };
        let refused = post(&app, "/api/search/window", window(west_evening_before.id)).await;
        if !ids(&refused["items"]).is_empty() {
            panic!("3.{minor} anchored on another day: {refused}");
        }
        let around = post(&app, "/api/search/window", window(morning_east.id)).await;
        if ids(&around["items"]) != vec![morning_east.id, early_east.id] {
            panic!("3.{minor} window was {around}");
        }
    }
}

#[tokio::test]
async fn a_long_span_trusts_its_middle_and_settles_both_ends() {
    let before_start = shot("2025-03-01T02:00:00Z", -5);
    let first_morning = shot("2025-03-01T05:00:00Z", 3);
    let middle = shot("2025-03-15T12:00:00Z", 0);
    let last_evening = shot("2025-03-31T20:00:00Z", 3);
    let past_end = shot("2025-03-31T22:00:00Z", 3);
    let photos = [before_start, first_morning, middle, last_evening, past_end];
    let march =
        json!({ "takenLocal": { "from": "2025-03-01T00:00:00", "until": "2025-04-01T00:00:00" } });
    for minor in [FLAT, STRUCTURED] {
        let (_server, app) = library(minor, &photos).await;
        let found = feed(&app, &march).await;
        if found != vec![last_evening.id, middle.id, first_morning.id] {
            panic!("3.{minor} listed {found:?}");
        }
    }
}
