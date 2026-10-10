use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{Direction, PageQuery, Sort, SortField};
use crate::services::search::{Searcher, added};

pub const MAX_RADIUS: u32 = 250;

type SortKey = (DateTime<Utc>, Uuid);

fn sort_key(asset: &AssetSummary) -> Option<SortKey> {
    let taken = DateTime::parse_from_rfc3339(asset.file_created_at.as_deref()?).ok()?;
    Some((taken.with_timezone(&Utc), asset.id.source()))
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
    if sort.field == SortField::Added {
        return added::around(searcher, filter, sort.direction, anchor, radius).await;
    }
    let Some(key) = sort_key(&anchor) else {
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
    let direction = match toward {
        Ordering::Greater => {
            bounded.taken_after = Some(filter.taken_after.map_or(anchor.0, |at| at.max(anchor.0)));
            Direction::Asc
        }
        _ => {
            bounded.taken_before =
                Some(filter.taken_before.map_or(anchor.0, |at| at.min(anchor.0)));
            Direction::Desc
        }
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
        out.extend(
            page.items
                .into_iter()
                .filter(|asset| sort_key(asset).is_some_and(|key| key.cmp(&anchor) == toward)),
        );
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
    use serde_json::Value;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::immich::ImmichClient;
    use crate::immich::client::ImmichAuth;
    use crate::services::search::SearchShape;

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
                    serde_json::json!({
                        "id": asset,
                        "type": "IMAGE",
                        "fileCreatedAt": taken.to_rfc3339_opts(SecondsFormat::Millis, true),
                    })
                })
                .collect();
            let next = (start + size < matched.len()).then(|| (page + 1).to_string());
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
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
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/search/metadata"))
            .respond_with(FakeSearch {
                assets: assets.clone(),
            })
            .mount(&server)
            .await;
        let immich = ImmichClient::with_auth(
            Url::parse(&server.uri()).unwrap(),
            ImmichAuth::ApiKey("key".into()),
            Duration::from_secs(5),
        )
        .unwrap();
        let searcher = Searcher::new(immich, SearchShape::Flat);
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
            let expected: Vec<Uuid> = feed
                [index.saturating_sub(radius)..(index + radius + 1).min(feed.len())]
                .iter()
                .map(|(_, id)| *id)
                .collect();
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
}
