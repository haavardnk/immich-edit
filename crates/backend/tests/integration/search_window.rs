mod library;

use crate::common::*;
use axum::http::StatusCode;
use chrono::{TimeDelta, TimeZone, Utc};
use library::{Key, Photo, feed, mount};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

type Outline = (Vec<String>, Value, Value);

fn photos(count: usize, tie: usize) -> Vec<Photo> {
    let base = Utc.with_ymd_and_hms(2024, 5, 1, 12, 0, 0).unwrap();
    (0..count)
        .map(|i| Photo {
            id: Uuid::new_v4(),
            taken: base + TimeDelta::seconds((i / tie) as i64),
            bytes: 1_000 * (i / tie) as u64,
        })
        .collect()
}

async fn library(minor: u64, photos: Vec<Photo>) -> (MockServer, axum::Router) {
    let server = MockServer::start().await;
    mock_server_version(&server, 3, minor, 2).await;
    mount(&server, photos).await;
    let app = test_app(&server).await;
    (server, app)
}

fn request(anchor: Uuid, field: &str, ascending: bool, reach: &Value) -> Value {
    let direction = if ascending { "asc" } else { "desc" };
    let mut body = json!({
        "assetId": anchor,
        "filter": {},
        "sort": { "field": field, "direction": direction },
    });
    if let (Some(body), Some(reach)) = (body.as_object_mut(), reach.as_object()) {
        body.extend(reach.clone());
    }
    body
}

async fn status(app: &axum::Router, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(json_request("POST", "/api/search/window", body))
        .await
        .unwrap();
    let status = resp.status();
    if status != StatusCode::OK {
        return (status, Value::Null);
    }
    (status, body_json(resp).await)
}

async fn window(app: &axum::Router, body: Value) -> Value {
    let (code, window) = status(app, body.clone()).await;
    if code != StatusCode::OK {
        panic!("window {body} gave {code}");
    }
    window
}

fn outline(window: &Value) -> Outline {
    let ids = window["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    (ids, window["hasBefore"].clone(), window["hasAfter"].clone())
}

fn expected(feed: &[Uuid], index: usize, before: usize, after: usize) -> Outline {
    let start = index.saturating_sub(before);
    let end = (index + after + 1).min(feed.len());
    let ids = feed[start..end].iter().map(Uuid::to_string).collect();
    (ids, json!(start > 0), json!(end < feed.len()))
}

async fn check(
    app: &axum::Router,
    library_photos: &[Photo],
    (field, key): (&str, Key),
    cases: &[(bool, usize, Value, usize, usize)],
    label: &str,
) {
    for (ascending, index, reach, before, after) in cases {
        let feed = feed(library_photos, key, *ascending);
        let body = request(feed[*index], field, *ascending, reach);
        let found = outline(&window(app, body).await);
        let wanted = expected(&feed, *index, *before, *after);
        if found != wanted {
            panic!(
                "{label} {field} asc={ascending} around {index} with {reach} gave {found:?}, expected {wanted:?}"
            );
        }
    }
}

#[tokio::test]
async fn taken_windows_reach_each_side_separately_on_both_shapes() {
    let library_photos = photos(12, 3);
    let cases = [
        (false, 6, json!({ "before": 2, "after": 4 }), 2, 4),
        (false, 6, json!({ "before": 0, "after": 3 }), 0, 3),
        (false, 6, json!({ "before": 4, "after": 0 }), 4, 0),
        (false, 0, json!({ "before": 5, "after": 2 }), 5, 2),
        (false, 11, json!({ "before": 2, "after": 5 }), 2, 5),
        (false, 3, json!({ "before": 3, "after": 8 }), 3, 8),
        (false, 5, json!({ "before": 0, "after": 0 }), 0, 0),
        (false, 3, json!({ "before": 0, "after": 1 }), 0, 1),
        (false, 4, json!({ "before": 1, "after": 0 }), 1, 0),
        (true, 2, json!({ "before": 1, "after": 3 }), 1, 3),
        (true, 10, json!({ "after": 4 }), 0, 4),
        (true, 1, json!({ "before": 3 }), 3, 0),
    ];
    for minor in [1, 2] {
        let (_server, app) = library(minor, library_photos.clone()).await;
        let label = format!("3.{minor}");
        check(&app, &library_photos, ("taken", Key::Taken), &cases, &label).await;
    }
}

#[tokio::test]
async fn size_windows_reach_each_side_separately() {
    let library_photos = photos(10, 2);
    let cases = [
        (false, 4, json!({ "before": 1, "after": 3 }), 1, 3),
        (false, 9, json!({ "before": 3, "after": 2 }), 3, 2),
        (false, 0, json!({ "before": 0, "after": 0 }), 0, 0),
        (true, 0, json!({ "before": 2, "after": 0 }), 2, 0),
        (true, 5, json!({ "radius": 2 }), 2, 2),
        (true, 4, json!({ "radius": 9, "before": 0 }), 0, 9),
    ];
    let (_server, app) = library(2, library_photos.clone()).await;
    check(&app, &library_photos, ("size", Key::Size), &cases, "3.2").await;
}

#[tokio::test]
async fn a_radius_alone_keeps_the_symmetric_window_and_adds_the_flags() {
    let library_photos = photos(9, 1);
    let cases = [
        (false, 4, json!({ "radius": 2 }), 2, 2),
        (false, 1, json!({ "radius": 3 }), 3, 3),
        (false, 4, json!({ "radius": 2, "after": 0 }), 2, 0),
        (true, 4, json!({ "radius": 250 }), 250, 250),
    ];
    for minor in [1, 2] {
        let (_server, app) = library(minor, library_photos.clone()).await;
        let label = format!("3.{minor}");
        check(&app, &library_photos, ("taken", Key::Taken), &cases, &label).await;
    }
}

#[tokio::test]
async fn an_anchor_outside_the_view_gives_an_empty_window() {
    let (_server, app) = library(2, photos(3, 1)).await;
    let body = request(Uuid::new_v4(), "taken", false, &json!({ "radius": 5 }));
    let found = window(&app, body).await;
    if found != json!({ "items": [], "hasBefore": false, "hasAfter": false }) {
        panic!("a missing anchor gave {found}");
    }
}

#[tokio::test]
async fn copies_follow_their_master_without_counting_towards_a_side() {
    let mut library_photos = photos(8, 1);
    library_photos[5].id = asset_id();
    let (server, app) = library(2, library_photos.clone()).await;
    mock_asset_detail(&server).await;
    for name in ["Warm", "Mono"] {
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/assets/{}/copies", asset_id()),
                json!({ "name": name }),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::CREATED {
            panic!("creating copy {name} gave {}", resp.status());
        }
    }
    let feed = feed(&library_photos, Key::Taken, false);
    if feed[2] != asset_id() {
        panic!("the master should sit third in the feed: {feed:?}");
    }
    let master = asset_id().to_string();
    let copies = [format!("{master}_1"), format!("{master}_2")];
    let cases = [
        (
            feed[4],
            json!({ "before": 2, "after": 0 }),
            vec![
                master.clone(),
                copies[0].clone(),
                copies[1].clone(),
                feed[3].to_string(),
                feed[4].to_string(),
            ],
        ),
        (
            feed[2],
            json!({ "before": 0, "after": 1 }),
            vec![
                master.clone(),
                copies[0].clone(),
                copies[1].clone(),
                feed[3].to_string(),
            ],
        ),
    ];
    for (anchor, reach, ids) in cases {
        let found = outline(&window(&app, request(anchor, "taken", false, &reach)).await);
        let wanted = (ids, json!(true), json!(true));
        if found != wanted {
            panic!("copies around {anchor} with {reach} gave {found:?}, expected {wanted:?}");
        }
    }
}

#[tokio::test]
async fn reaches_out_of_range_are_rejected() {
    let library_photos = photos(3, 1);
    let anchor = library_photos[0].id;
    let (_server, app) = library(2, library_photos).await;
    let cases = [
        (json!({}), StatusCode::BAD_REQUEST),
        (json!({ "radius": null }), StatusCode::BAD_REQUEST),
        (json!({ "radius": 0 }), StatusCode::BAD_REQUEST),
        (json!({ "radius": 251 }), StatusCode::BAD_REQUEST),
        (
            json!({ "radius": 251, "before": 1, "after": 1 }),
            StatusCode::BAD_REQUEST,
        ),
        (json!({ "before": 501 }), StatusCode::BAD_REQUEST),
        (
            json!({ "radius": 5, "after": 501 }),
            StatusCode::BAD_REQUEST,
        ),
        (json!({ "before": -1 }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "limit": 5 }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "before": 500, "after": 500 }), StatusCode::OK),
        (json!({ "before": 0, "after": 0 }), StatusCode::OK),
    ];
    for (reach, code) in cases {
        let (found, _) = status(&app, request(anchor, "taken", false, &reach)).await;
        if found != code {
            panic!("{reach} gave {found}, expected {code}");
        }
    }
}
