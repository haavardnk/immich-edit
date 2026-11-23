use std::time::Duration;

use chrono::{SecondsFormat, TimeDelta, TimeZone};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use super::*;
use crate::immich::ImmichClient;
use crate::immich::client::ImmichAuth;
use crate::services::search::SearchShape;

type Shot = (DateTime<Utc>, Uuid);

#[derive(Clone)]
struct FakeLibrary {
    shots: Vec<Shot>,
}

impl FakeLibrary {
    fn matching(&self, body: &Value) -> Vec<Shot> {
        let bound = |name: &str| {
            body.get(name)
                .and_then(Value::as_str)
                .map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
        };
        let after = bound("takenAfter");
        let before = bound("takenBefore");
        let mut matched: Vec<Shot> = self
            .shots
            .iter()
            .filter(|(taken, _)| after.is_none_or(|a| *taken >= a))
            .filter(|(taken, _)| before.is_none_or(|b| *taken <= b))
            .copied()
            .collect();
        matched.sort();
        if body.get("order").and_then(Value::as_str) != Some("asc") {
            matched.reverse();
        }
        matched
    }
}

struct Statistics(FakeLibrary);

impl Respond for Statistics {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = request.body_json().unwrap();
        ResponseTemplate::new(200).set_body_json(json!({ "total": self.0.matching(&body).len() }))
    }
}

struct Metadata(FakeLibrary);

impl Respond for Metadata {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = request.body_json().unwrap();
        let matched = self.0.matching(&body);
        let size = (body["size"].as_u64().unwrap() as usize).min(40);
        let page = body.get("page").and_then(Value::as_u64).unwrap_or(1) as usize;
        let start = (page - 1) * size;
        let items: Vec<Value> = matched
            .iter()
            .skip(start)
            .take(size)
            .map(|(taken, id)| {
                json!({
                    "id": id,
                    "type": "IMAGE",
                    "fileCreatedAt": taken.to_rfc3339_opts(SecondsFormat::Millis, true),
                })
            })
            .collect();
        let next = (start + size < matched.len()).then(|| (page + 1).to_string());
        ResponseTemplate::new(200).set_body_json(json!({
            "assets": { "items": items, "count": items.len(), "total": matched.len(), "nextPage": next },
        }))
    }
}

async fn searcher(library: FakeLibrary) -> (MockServer, Searcher) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .respond_with(Statistics(library.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .respond_with(Metadata(library))
        .mount(&server)
        .await;
    let immich = ImmichClient::with_auth(
        Url::parse(&server.uri()).unwrap(),
        ImmichAuth::ApiKey("key".into()),
        Duration::from_secs(5),
        None,
    )
    .unwrap();
    (server, Searcher::new(immich, SearchShape::Flat))
}

fn march_library() -> Vec<Shot> {
    let burst = Utc.with_ymd_and_hms(2024, 3, 9, 14, 0, 0).unwrap();
    let spread = Utc.with_ymd_and_hms(2024, 3, 2, 8, 0, 0).unwrap();
    let mut shots: Vec<Shot> = (0..320)
        .map(|_| (burst, Uuid::new_v4()))
        .chain((0..300).map(|i| (burst + TimeDelta::milliseconds(i * 7), Uuid::new_v4())))
        .chain((0..260).map(|i| (spread + TimeDelta::hours(i), Uuid::new_v4())))
        .collect();
    shots.push((
        Utc.with_ymd_and_hms(2024, 2, 29, 23, 59, 59).unwrap(),
        Uuid::new_v4(),
    ));
    shots.push((
        Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
        Uuid::new_v4(),
    ));
    shots
}

fn query(direction: Direction, month: &str, offset: u64) -> LocateQuery {
    LocateQuery {
        filter: BrowseFilter::default(),
        direction,
        month: month.into(),
        offset,
    }
}

#[tokio::test]
async fn every_offset_of_a_month_finds_the_photo_at_that_place_in_the_feed() {
    let shots = march_library();
    let (_server, searcher) = searcher(FakeLibrary {
        shots: shots.clone(),
    })
    .await;
    let start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap();
    let mut march: Vec<Shot> = shots
        .into_iter()
        .filter(|(taken, _)| *taken >= start && *taken < end)
        .collect();
    march.sort();
    for direction in [Direction::Asc, Direction::Desc] {
        let mut feed = march.clone();
        if direction == Direction::Desc {
            feed.reverse();
        }
        for offset in (0..feed.len()).step_by(13).chain([feed.len() - 1]) {
            let found = photo_at(&searcher, &query(direction, "2024-03", offset as u64))
                .await
                .unwrap()
                .asset_id
                .map(|id| id.source());
            if found != Some(feed[offset].1) {
                panic!("{direction:?} offset {offset} found {found:?}");
            }
        }
        let past_the_end = photo_at(&searcher, &query(direction, "2024-03", 99_999))
            .await
            .unwrap()
            .asset_id
            .map(|id| id.source());
        if past_the_end != feed.last().map(|(_, id)| *id) {
            panic!("{direction:?} offset past the end found {past_the_end:?}");
        }
    }
}

#[tokio::test]
async fn an_empty_month_finds_nothing_and_a_malformed_month_is_rejected() {
    let (_server, searcher) = searcher(FakeLibrary {
        shots: march_library(),
    })
    .await;
    let empty = photo_at(&searcher, &query(Direction::Desc, "2023-07", 0))
        .await
        .unwrap();
    if empty.asset_id.is_some() {
        panic!("an empty month found {empty:?}");
    }
    let malformed = photo_at(&searcher, &query(Direction::Desc, "2024-3", 0)).await;
    if !matches!(malformed, Err(AppError::BadRequest(_))) {
        panic!("a malformed month gave {malformed:?}");
    }
}
