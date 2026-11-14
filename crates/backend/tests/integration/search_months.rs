use std::time::Duration;

use crate::common::search_library::{ALBUM, Photo, Shelf, TAG, mount, mount_slow};
use crate::common::*;
use axum::http::StatusCode;
use chrono::{DateTime, Days, NaiveDate, Utc};
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::path_regex;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn at(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .unwrap()
        .with_timezone(&Utc)
}

fn photos(raws: &[&str]) -> Vec<Photo> {
    raws.iter()
        .map(|raw| Photo {
            taken: at(raw),
            ..Photo::default()
        })
        .collect()
}

async fn library(minor: u64, photos: Vec<Photo>) -> (MockServer, axum::Router, Shelf) {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, minor, 2).await;
    let shelf = mount(&server, TEST_API_KEY, photos).await;
    let app = test_app(&server).await;
    (server, app, shelf)
}

async fn months(app: &axum::Router, filter: Value) -> Value {
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/search/months",
            json!({ "filter": filter }),
        ))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("months for {filter} gave {}", resp.status());
    }
    body_json(resp).await
}

async fn search_calls(server: &MockServer) -> (usize, usize) {
    let requests = server.received_requests().await.unwrap_or_default();
    let calls = |route: &str| {
        requests
            .iter()
            .filter(|request| request.url.path() == route)
            .count()
    };
    (
        calls("/api/search/metadata"),
        calls("/api/search/statistics"),
    )
}

fn single(month: &str, count: u64) -> Value {
    json!({ "months": [{ "month": month, "count": count }] })
}

async fn settle(server: &MockServer, at_least: (usize, usize)) -> (usize, usize) {
    for _ in 0..500 {
        let calls = search_calls(server).await;
        if calls.0 >= at_least.0 && calls.1 >= at_least.1 {
            return calls;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("search calls never reached {at_least:?}");
}

#[tokio::test]
async fn an_empty_library_has_no_months_after_one_count() {
    for minor in [1, 2] {
        let (server, app, _) = library(minor, Vec::new()).await;
        let counts = months(&app, json!({})).await;
        let calls = search_calls(&server).await;
        if counts != json!({ "months": [] }) || calls != (0, 1) {
            panic!("3.{minor} empty library gave {counts} after {calls:?} calls");
        }
    }
}

#[tokio::test]
async fn one_month_is_counted_once() {
    let taken = [
        "2024-07-02T10:00:00.000Z",
        "2024-07-15T08:30:00.250Z",
        "2024-07-31T21:00:00.000Z",
    ];
    for minor in [1, 2] {
        let (server, app, _) = library(minor, photos(&taken)).await;
        let counts = months(&app, json!({})).await;
        let calls = search_calls(&server).await;
        if counts != single("2024-07", 3) || calls != (2, 2) {
            panic!("3.{minor} one month gave {counts} after {calls:?} calls");
        }
    }
}

#[tokio::test]
async fn years_without_photos_are_skipped_before_months_are_counted() {
    let taken = [
        "2019-11-05T12:00:00.000Z",
        "2019-11-20T12:00:00.000Z",
        "2021-02-14T12:00:00.000Z",
        "2021-12-31T12:00:00.000Z",
        "2023-06-01T12:00:00.000Z",
    ];
    let expected = json!({ "months": [
        { "month": "2023-06", "count": 1 },
        { "month": "2021-12", "count": 1 },
        { "month": "2021-02", "count": 1 },
        { "month": "2019-11", "count": 2 },
    ] });
    for minor in [1, 2] {
        let (server, app, _) = library(minor, photos(&taken)).await;
        let counts = months(&app, json!({})).await;
        let calls = search_calls(&server).await;
        let total_call = 1;
        let year_calls = 3;
        let month_calls = 2 + 12 + 6;
        if counts != expected || calls != (2, total_call + year_calls + month_calls) {
            panic!("3.{minor} gapped library gave {counts} after {calls:?} calls");
        }
    }
}

#[tokio::test]
async fn the_filter_and_its_taken_range_clamp_the_months() {
    let favorite = |raw: &str| Photo {
        taken: at(raw),
        favorite: true,
        ..Photo::default()
    };
    let library_photos = vec![
        favorite("2024-01-20T12:00:00.000Z"),
        favorite("2024-02-10T12:00:00.000Z"),
        favorite("2024-02-20T12:00:00.000Z"),
        Photo {
            taken: at("2024-03-10T12:00:00.000Z"),
            ..Photo::default()
        },
        favorite("2024-04-05T12:00:00.000Z"),
        favorite("2024-04-20T12:00:00.000Z"),
    ];
    let filter = json!({
        "favorite": true,
        "takenAfter": "2024-02-15T00:00:00Z",
        "takenBefore": "2024-04-10T00:00:00Z",
    });
    let expected = json!({ "months": [
        { "month": "2024-04", "count": 1 },
        { "month": "2024-02", "count": 1 },
    ] });
    for minor in [1, 2] {
        let (_server, app, _) = library(minor, library_photos.clone()).await;
        let counts = months(&app, filter.clone()).await;
        if counts != expected {
            panic!("3.{minor} clamped months were {counts}");
        }
    }
}

#[tokio::test]
async fn an_asset_on_a_month_boundary_lands_in_exactly_one_month() {
    let flat_taken = [
        "2024-03-31T23:59:59.999Z",
        "2024-04-01T00:00:00.000Z",
        "2024-04-01T00:00:00.001Z",
    ];
    let structured_taken = [
        "2024-03-31T23:59:59.999Z",
        "2024-03-31T23:59:59.999500Z",
        "2024-04-01T00:00:00.000Z",
        "2024-04-01T00:00:00.000500Z",
    ];
    for (minor, taken, march, april) in
        [(1, &flat_taken[..], 1, 2), (2, &structured_taken[..], 2, 2)]
    {
        let (_server, app, _) = library(minor, photos(taken)).await;
        let counts = months(&app, json!({})).await;
        let expected = json!({ "months": [
            { "month": "2024-04", "count": april },
            { "month": "2024-03", "count": march },
        ] });
        if counts != expected {
            panic!("3.{minor} boundary months were {counts}");
        }
    }
}

#[tokio::test]
async fn month_counts_add_up_to_the_library_total() {
    let start = NaiveDate::from_ymd_opt(2022, 12, 1).unwrap();
    let library_photos: Vec<Photo> = (0..400)
        .map(|day| Photo {
            taken: (start + Days::new(day))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            ..Photo::default()
        })
        .collect();
    for minor in [1, 2] {
        let (_server, app, _) = library(minor, library_photos.clone()).await;
        let counts = months(&app, json!({})).await;
        let listed = counts["months"].as_array().cloned().unwrap_or_default();
        let total: u64 = listed
            .iter()
            .filter_map(|month| month["count"].as_u64())
            .sum();
        let newest_first = listed
            .windows(2)
            .all(|pair| pair[0]["month"].as_str() > pair[1]["month"].as_str());
        if total != 400 || listed.len() != 14 || !newest_first {
            panic!("3.{minor} daily library gave {counts}");
        }
    }
}

#[tokio::test]
async fn a_repeated_view_is_served_from_the_cache_after_one_count() {
    let taken = ["2022-05-01T00:00:00.000Z", "2024-05-01T00:00:00.000Z"];
    for minor in [1, 2] {
        let (server, app, _) = library(minor, photos(&taken)).await;
        let first = months(&app, json!({})).await;
        let (metadata, statistics) = search_calls(&server).await;
        let second = months(&app, json!({})).await;
        let after = search_calls(&server).await;
        if first != second || after != (metadata, statistics + 2) {
            panic!(
                "3.{minor} repeat gave {second} with calls {:?} then {after:?}",
                (metadata, statistics)
            );
        }
        let favorites = months(&app, json!({ "favorite": true })).await;
        let other = search_calls(&server).await;
        if favorites != json!({ "months": [] }) || other.1 == after.1 {
            panic!("3.{minor} another filter reused the cache: {favorites} {other:?}");
        }
    }
}

#[tokio::test]
async fn a_cached_view_recounts_after_the_library_changes() {
    let favorite = |raw: &str, favorite: bool| Photo {
        taken: at(raw),
        favorite,
        ..Photo::default()
    };
    for minor in [1, 2] {
        let (_server, app, shelf) = library(
            minor,
            vec![
                favorite("2024-01-10T00:00:00.000Z", true),
                favorite("2024-03-10T00:00:00.000Z", false),
            ],
        )
        .await;
        let filter = json!({ "favorite": true });
        let before = months(&app, filter.clone()).await;
        shelf.lock().unwrap()[1].favorite = true;
        let favorited = months(&app, filter.clone()).await;
        shelf.lock().unwrap().remove(0);
        let deleted = months(&app, filter.clone()).await;
        let expected_favorited = json!({ "months": [
            { "month": "2024-03", "count": 1 },
            { "month": "2024-01", "count": 1 },
        ] });
        if before != single("2024-01", 1)
            || favorited != expected_favorited
            || deleted != single("2024-03", 1)
        {
            panic!("3.{minor} recount gave {before} then {favorited} then {deleted}");
        }
    }
}

#[tokio::test]
async fn a_date_edit_that_keeps_the_total_recounts() {
    for minor in [1, 2] {
        let (_server, app, shelf) = library(
            minor,
            photos(&["2024-01-10T00:00:00.000Z", "2024-03-10T00:00:00.000Z"]),
        )
        .await;
        let before = months(&app, json!({})).await;
        {
            let mut photos = shelf.lock().unwrap();
            photos[1].taken = at("2024-01-20T00:00:00.000Z");
            photos[1].updated = Utc::now();
        }
        let moved = months(&app, json!({})).await;
        let expected_before = json!({ "months": [
            { "month": "2024-03", "count": 1 },
            { "month": "2024-01", "count": 1 },
        ] });
        if before != expected_before || moved != single("2024-01", 2) {
            panic!("3.{minor} date edit gave {before} then {moved}");
        }
    }
}

async fn swap_membership_through(
    route: (&str, &str),
    filter: Value,
    membership: fn(&mut Photo) -> &mut bool,
) {
    for minor in [1, 2] {
        let mut start = photos(&["2024-01-10T00:00:00.000Z", "2024-03-10T00:00:00.000Z"]);
        *membership(&mut start[0]) = true;
        let (server, app, shelf) = library(minor, start).await;
        Mock::given(path_regex(r"^/api/(albums|tags)/[^/]+/assets$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;
        let before = months(&app, filter.clone()).await;
        {
            let mut photos = shelf.lock().unwrap();
            *membership(&mut photos[0]) = false;
            *membership(&mut photos[1]) = true;
        }
        let (verb, uri) = route;
        let resp = app
            .clone()
            .oneshot(json_request(
                verb,
                uri,
                json!({ "ids": [uuid::Uuid::nil()] }),
            ))
            .await
            .unwrap();
        let status = resp.status();
        let after = months(&app, filter.clone()).await;
        if before != single("2024-01", 1)
            || status != StatusCode::OK
            || after != single("2024-03", 1)
        {
            panic!("3.{minor} {verb} {uri} gave {before} then {status} {after}");
        }
    }
}

#[tokio::test]
async fn an_album_membership_change_recounts_the_album_months() {
    let uri = format!("/api/albums/{ALBUM}/assets");
    let filter = json!({ "albums": { "all": [ALBUM] } });
    for verb in ["PUT", "DELETE"] {
        swap_membership_through((verb, &uri), filter.clone(), |photo| &mut photo.in_album).await;
    }
}

#[tokio::test]
async fn a_tag_change_recounts_the_tag_months() {
    let uri = format!("/api/tags/{TAG}/assets/{}", uuid::Uuid::nil());
    let filter = json!({ "tags": { "all": [TAG] } });
    for verb in ["PUT", "DELETE"] {
        swap_membership_through((verb, &uri), filter.clone(), |photo| &mut photo.tagged).await;
    }
}

#[tokio::test]
async fn concurrent_identical_views_share_one_count() {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, 2, 2).await;
    let taken = photos(&["2024-07-02T10:00:00.000Z"]);
    mount_slow(&server, TEST_API_KEY, taken, Duration::from_millis(150)).await;
    let app = test_app(&server).await;
    months(&app, json!({ "favorite": true })).await;
    let warmed = search_calls(&server).await;
    let (first, second) = tokio::join!(months(&app, json!({})), months(&app, json!({})));
    let calls = search_calls(&server).await;
    let shared = (calls.0 - warmed.0, calls.1 - warmed.1);
    if first != single("2024-07", 1) || first != second || shared != (2, 2) {
        panic!("concurrent views gave {first} and {second} after {shared:?} calls");
    }
}

#[tokio::test]
async fn a_dropped_request_still_fills_the_cache() {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, 2, 2).await;
    let taken = photos(&["2024-07-02T10:00:00.000Z"]);
    mount_slow(&server, TEST_API_KEY, taken, Duration::from_millis(100)).await;
    let app = test_app(&server).await;
    months(&app, json!({ "favorite": true })).await;
    let request = tokio::spawn(app.clone().oneshot(json_request(
        "POST",
        "/api/search/months",
        json!({ "filter": {} }),
    )));
    settle(&server, (0, 2)).await;
    if request.is_finished() {
        panic!("the slow request finished before it was dropped");
    }
    request.abort();
    let filled = settle(&server, (2, 3)).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    let counts = months(&app, json!({})).await;
    let after = search_calls(&server).await;
    if counts != single("2024-07", 1) || filled != (2, 3) || after != (2, 5) {
        panic!("dropped request left {filled:?}, then {counts} after {after:?}");
    }
}

#[tokio::test]
async fn owners_with_the_same_filter_see_their_own_months() {
    for minor in [1, 2] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        let admin_photos = photos(&["2024-01-10T00:00:00.000Z", "2024-01-20T00:00:00.000Z"]);
        let member_photos = photos(&["2023-05-10T00:00:00.000Z", "2023-05-20T00:00:00.000Z"]);
        mount(&server, TEST_API_KEY, admin_photos).await;
        mount(&server, MEMBER_API_KEY, member_photos).await;
        let (admin, member) = two_owner_apps(&server).await;
        let admin_months = months(&admin, json!({})).await;
        let member_months = months(&member, json!({})).await;
        if admin_months != single("2024-01", 2) || member_months != single("2023-05", 2) {
            panic!("3.{minor} owners saw {admin_months} and {member_months}");
        }
    }
}

#[tokio::test]
async fn unknown_fields_are_rejected() {
    let (_server, app, _) = library(2, Vec::new()).await;
    for body in [
        json!({ "filter": {}, "sort": { "field": "taken" } }),
        json!({ "filter": { "takenUntil": "2024-01-01T00:00:00Z" } }),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/search/months", body.clone()))
            .await
            .unwrap();
        if resp.status() != StatusCode::UNPROCESSABLE_ENTITY {
            panic!("{body} gave {}", resp.status());
        }
    }
}
