use std::time::Duration;

use chrono::{SecondsFormat, TimeZone};
use serde_json::{Value, json};
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use super::*;
use crate::immich::ImmichClient;
use crate::immich::client::ImmichAuth;
use crate::services::search::SearchShape;

pub(in crate::services::search) type Outline = (Vec<Uuid>, bool, bool);

pub(in crate::services::search) fn expected_window<T>(
    feed: &[(T, Uuid)],
    index: usize,
    reach: Reach,
) -> Outline {
    let start = index.saturating_sub(reach.before as usize);
    let end = (index + reach.after as usize + 1).min(feed.len());
    let ids = feed[start..end].iter().map(|(_, id)| *id).collect();
    (ids, start > 0, end < feed.len())
}

pub(in crate::services::search) fn outline(window: &Window) -> Outline {
    let ids = window.items.iter().map(|asset| asset.id.source()).collect();
    (ids, window.has_before, window.has_after)
}

pub(in crate::services::search) fn reaches() -> [Reach; 7] {
    [(5, 5), (2, 7), (7, 2), (0, 4), (3, 0), (0, 0), (40, 40)]
        .map(|(before, after)| Reach { before, after })
}

async fn searcher(fake: impl Respond + 'static, shape: SearchShape) -> (MockServer, Searcher) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .respond_with(fake)
        .mount(&server)
        .await;
    let immich = ImmichClient::with_auth(
        Url::parse(&server.uri()).unwrap(),
        ImmichAuth::ApiKey("key".into()),
        Duration::from_secs(5),
        None,
    )
    .unwrap();
    (server, Searcher::new(immich, shape))
}

async fn check_every_anchor<T: Clone>(
    searcher: &Searcher,
    filter: &BrowseFilter,
    field: SortField,
    sorted: &[(T, Uuid)],
) {
    for direction in [Direction::Desc, Direction::Asc] {
        let mut feed = sorted.to_vec();
        if direction == Direction::Desc {
            feed.reverse();
        }
        let sort = Sort { field, direction };
        for index in 0..feed.len() {
            for reach in reaches() {
                let window = around(searcher, filter, sort, feed[index].1, reach)
                    .await
                    .unwrap();
                let found = outline(&window);
                let expected = expected_window(&feed, index, reach);
                if found != expected {
                    panic!(
                        "{field:?} {direction:?} window around {index} with {reach:?} was {found:?}, expected {expected:?}"
                    );
                }
            }
        }
    }
}

struct FakeSearch {
    assets: Vec<(DateTime<Utc>, Uuid)>,
}

impl Respond for FakeSearch {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = request.body_json().unwrap();
        let bound = |name: &str| {
            body.get(name)
                .and_then(Value::as_str)
                .map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
        };
        let after = bound("takenAfter");
        let before = bound("takenBefore");
        let id = body.get("id").and_then(Value::as_str);
        let mut matched: Vec<_> = self
            .assets
            .iter()
            .filter(|(taken, _)| after.is_none_or(|a| *taken >= a))
            .filter(|(taken, _)| before.is_none_or(|b| *taken <= b))
            .filter(|(_, asset)| id.is_none_or(|i| asset.to_string() == i))
            .copied()
            .collect();
        matched.sort();
        if body.get("order").and_then(Value::as_str) != Some("asc") {
            matched.reverse();
        }
        let size = body["size"].as_u64().unwrap() as usize;
        let page = body.get("page").and_then(Value::as_u64).unwrap_or(1) as usize;
        let start = (page - 1) * size;
        let items: Vec<Value> = matched
            .iter()
            .skip(start)
            .take(size)
            .map(|(taken, asset)| {
                json!({
                    "id": asset,
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

#[tokio::test]
async fn taken_windows_match_the_feed_for_every_anchor_reach_and_tie() {
    let base = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let assets: Vec<(DateTime<Utc>, Uuid)> = (0..30)
        .map(|i| (base + chrono::Duration::seconds(i / 4), Uuid::new_v4()))
        .collect();
    let (_server, searcher) = searcher(
        FakeSearch {
            assets: assets.clone(),
        },
        SearchShape::Flat,
    )
    .await;
    let filter = BrowseFilter::default();
    let mut sorted = assets;
    sorted.sort();
    check_every_anchor(&searcher, &filter, SortField::Taken, &sorted).await;
    let missing = around(
        &searcher,
        &filter,
        Sort::default(),
        Uuid::new_v4(),
        reaches()[0],
    )
    .await
    .unwrap();
    if outline(&missing) != (Vec::new(), false, false) {
        panic!("an anchor outside the feed gave {missing:?}");
    }
}

struct FakeSizeSearch {
    assets: Vec<(u64, Uuid)>,
}

impl Respond for FakeSizeSearch {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = request.body_json().unwrap();
        let filter = &body["filter"];
        if body.get("orderBy").is_some() && body["orderBy"]["field"] != "fileSizeInBytes" {
            panic!("size window ordered by {}", body["orderBy"]);
        }
        let id = filter["id"]["eq"].as_str();
        let min = filter["fileSizeInBytes"]["gte"].as_u64();
        let max = filter["fileSizeInBytes"]["lte"].as_u64();
        let mut matched: Vec<(u64, Uuid)> = self
            .assets
            .iter()
            .filter(|(_, asset)| id.is_none_or(|i| asset.to_string() == i))
            .filter(|(bytes, _)| min.is_none_or(|m| *bytes >= m))
            .filter(|(bytes, _)| max.is_none_or(|m| *bytes <= m))
            .copied()
            .collect();
        matched.sort();
        if body["orderBy"]["direction"] != "asc" {
            matched.reverse();
        }
        let size = (body["size"].as_u64().unwrap() as usize).min(7);
        let start: usize = body["cursor"].as_str().map_or(0, |c| c.parse().unwrap());
        let items: Vec<Value> = matched
            .iter()
            .skip(start)
            .take(size)
            .map(|(bytes, asset)| {
                json!({ "id": asset, "type": "IMAGE", "exifInfo": { "fileSizeInByte": bytes } })
            })
            .collect();
        let next = (start + size < matched.len()).then(|| (start + size).to_string());
        ResponseTemplate::new(200).set_body_json(json!({
            "assets": { "items": items, "count": items.len(), "total": matched.len(), "nextCursor": next },
        }))
    }
}

#[tokio::test]
async fn size_windows_match_the_feed_for_every_anchor_reach_and_tie() {
    let assets: Vec<(u64, Uuid)> = (0..40).map(|i| (1_000 * (i / 3), Uuid::new_v4())).collect();
    let (_server, searcher) = searcher(
        FakeSizeSearch {
            assets: assets.clone(),
        },
        SearchShape::Structured,
    )
    .await;
    let filter = BrowseFilter {
        min_size: Some(2_000),
        ..BrowseFilter::default()
    };
    let mut sorted: Vec<(u64, Uuid)> = assets
        .into_iter()
        .filter(|(bytes, _)| *bytes >= 2_000)
        .collect();
    sorted.sort();
    check_every_anchor(&searcher, &filter, SortField::Size, &sorted).await;
}
