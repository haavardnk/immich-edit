use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{Direction, PageQuery, Sort, SortField};
use crate::services::search::{Searcher, added, rating};

pub const MAX_RADIUS: u32 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Position {
    Taken(DateTime<Utc>),
    Size(u64),
}

type SortKey = (Position, Uuid);

fn sort_key(field: SortField, asset: &AssetSummary) -> Option<SortKey> {
    let position = if field == SortField::Size {
        Position::Size(asset.exif_info.as_ref()?.file_size_in_byte?)
    } else {
        let taken = DateTime::parse_from_rfc3339(asset.file_created_at.as_deref()?).ok()?;
        Position::Taken(taken.with_timezone(&Utc))
    };
    Some((position, asset.id.source()))
}

pub async fn around(
    searcher: &Searcher,
    filter: &BrowseFilter,
    sort: Sort,
    anchor: Uuid,
    radius: u32,
) -> Result<Vec<AssetSummary>, AppError> {
    let Some(anchor) = searcher.find(filter, anchor).await? else {
        return Ok(Vec::new());
    };
    match sort.field {
        SortField::Added => {
            return added::around(searcher, filter, sort.direction, anchor, radius).await;
        }
        SortField::Rating => {
            return rating::around(searcher, filter, sort.direction, anchor, radius).await;
        }
        SortField::Taken | SortField::Size => {}
    }
    let Some(key) = sort_key(sort.field, &anchor) else {
        return Ok(vec![anchor]);
    };
    let newer = side(searcher, filter, sort, key, Ordering::Greater, radius).await?;
    let older = side(searcher, filter, sort, key, Ordering::Less, radius).await?;
    let (before, after) = match sort.direction {
        Direction::Desc => (newer, older),
        Direction::Asc => (older, newer),
    };
    Ok(before
        .into_iter()
        .rev()
        .chain(std::iter::once(anchor))
        .chain(after)
        .collect())
}

async fn side(
    searcher: &Searcher,
    filter: &BrowseFilter,
    sort: Sort,
    anchor: SortKey,
    toward: Ordering,
    radius: u32,
) -> Result<Vec<AssetSummary>, AppError> {
    let mut bounded = filter.clone();
    match (anchor.0, toward) {
        (Position::Taken(at), Ordering::Greater) => {
            bounded.taken_after = Some(filter.taken_after.map_or(at, |after| after.max(at)));
        }
        (Position::Taken(at), _) => {
            bounded.taken_before = Some(filter.taken_before.map_or(at, |before| before.min(at)));
        }
        (Position::Size(bytes), Ordering::Greater) => {
            bounded.min_size = Some(filter.min_size.map_or(bytes, |min| min.max(bytes)));
        }
        (Position::Size(bytes), _) => {
            bounded.max_size = Some(filter.max_size.map_or(bytes, |max| max.min(bytes)));
        }
    }
    let direction = match toward {
        Ordering::Greater => Direction::Asc,
        _ => Direction::Desc,
    };
    let mut query = PageQuery {
        filter: bounded,
        sort: Sort { direction, ..sort },
        size: radius + 1,
        cursor: None,
        with_exif: true,
    };
    let wanted = radius as usize;
    let mut out: Vec<AssetSummary> = Vec::new();
    loop {
        let page = searcher.page(&query).await?;
        out.extend(page.items.into_iter().filter(|asset| {
            sort_key(sort.field, asset).is_some_and(|key| key.cmp(&anchor) == toward)
        }));
        if out.len() >= wanted {
            break;
        }
        let Some(cursor) = page.next_cursor else {
            break;
        };
        query.cursor = Some(cursor);
    }
    out.truncate(wanted);
    Ok(out)
}

#[cfg(test)]
mod tests {
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
        )
        .unwrap();
        (server, Searcher::new(immich, shape))
    }

    fn expected_window<T>(feed: &[(T, Uuid)], index: usize, radius: usize) -> Vec<Uuid> {
        feed[index.saturating_sub(radius)..(index + radius + 1).min(feed.len())]
            .iter()
            .map(|(_, id)| *id)
            .collect()
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
    async fn window_matches_the_full_feed_around_the_anchor() {
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
        let mut sorted = assets.clone();
        sorted.sort();
        for (direction, index) in [
            (Direction::Desc, 0),
            (Direction::Desc, 13),
            (Direction::Desc, 29),
            (Direction::Asc, 5),
            (Direction::Asc, 29),
        ] {
            let mut feed = sorted.clone();
            if direction == Direction::Desc {
                feed.reverse();
            }
            let radius = 5;
            let anchor = feed[index].1;
            let expected = expected_window(&feed, index, radius);
            let sort = Sort {
                direction,
                ..Sort::default()
            };
            let window = around(&searcher, &filter, sort, anchor, radius as u32)
                .await
                .unwrap();
            let ids: Vec<Uuid> = window.iter().map(|asset| asset.id.source()).collect();
            if ids != expected {
                panic!("{direction:?} window around {index} was {ids:?}, expected {expected:?}");
            }
        }
        let missing = around(&searcher, &filter, Sort::default(), Uuid::new_v4(), 5)
            .await
            .unwrap();
        if !missing.is_empty() {
            panic!("an anchor outside the feed should give an empty window");
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
    async fn size_window_matches_the_full_feed_around_the_anchor() {
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
        for (direction, index) in [
            (Direction::Desc, 0),
            (Direction::Desc, 17),
            (Direction::Asc, 3),
            (Direction::Asc, 33),
        ] {
            let mut feed = sorted.clone();
            if direction == Direction::Desc {
                feed.reverse();
            }
            let radius = 9;
            let sort = Sort {
                field: SortField::Size,
                direction,
            };
            let window = around(&searcher, &filter, sort, feed[index].1, radius as u32)
                .await
                .unwrap();
            let ids: Vec<Uuid> = window.iter().map(|asset| asset.id.source()).collect();
            let expected = expected_window(&feed, index, radius);
            if ids != expected {
                panic!(
                    "{direction:?} size window around {index} was {ids:?}, expected {expected:?}"
                );
            }
        }
    }
}
