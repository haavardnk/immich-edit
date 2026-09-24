use std::cmp::Ordering;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::immich::dto::AssetSummary;
use crate::immich::{ImmichClient, ImmichError};

pub const MAX_RADIUS: u32 = 250;

type SortKey = (DateTime<Utc>, Uuid);

fn sort_key(asset: &AssetSummary) -> Option<SortKey> {
    let taken = DateTime::parse_from_rfc3339(asset.file_created_at.as_deref()?).ok()?;
    Some((taken.with_timezone(&Utc), asset.id.source()))
}

pub async fn around(
    immich: &ImmichClient,
    mut query: Map<String, Value>,
    anchor: Uuid,
    radius: u32,
) -> Result<Vec<AssetSummary>, ImmichError> {
    query.remove("page");
    let descending = query.get("order").and_then(Value::as_str) != Some("asc");
    let mut probe = query.clone();
    probe.insert("id".into(), anchor.to_string().into());
    probe.insert("size".into(), 1.into());
    let found = immich.search_metadata(&Value::Object(probe)).await?;
    let Some(anchor) = found.items.into_iter().next() else {
        return Ok(Vec::new());
    };
    let Some(key) = sort_key(&anchor) else {
        return Ok(vec![anchor]);
    };
    query.remove("id");
    let newer = side(immich, &query, key, Ordering::Greater, radius).await?;
    let older = side(immich, &query, key, Ordering::Less, radius).await?;
    let (before, after) = match descending {
        true => (newer, older),
        false => (older, newer),
    };
    Ok(before
        .into_iter()
        .rev()
        .chain(std::iter::once(anchor))
        .chain(after)
        .collect())
}

async fn side(
    immich: &ImmichClient,
    query: &Map<String, Value>,
    anchor: SortKey,
    toward: Ordering,
    radius: u32,
) -> Result<Vec<AssetSummary>, ImmichError> {
    let (bound, order) = match toward {
        Ordering::Greater => ("takenAfter", "asc"),
        _ => ("takenBefore", "desc"),
    };
    let mut body = query.clone();
    body.insert(
        bound.into(),
        anchor.0.to_rfc3339_opts(SecondsFormat::Millis, true).into(),
    );
    body.insert("order".into(), order.into());
    body.insert("size".into(), (radius + 1).into());
    let wanted = radius as usize;
    let mut out: Vec<AssetSummary> = Vec::new();
    let mut page = 1u32;
    loop {
        body.insert("page".into(), page.into());
        let result = immich.search_metadata(&Value::Object(body.clone())).await?;
        out.extend(
            result
                .items
                .into_iter()
                .filter(|asset| sort_key(asset).is_some_and(|key| key.cmp(&anchor) == toward)),
        );
        if out.len() >= wanted || result.next_page.is_none() {
            break;
        }
        page += 1;
    }
    out.truncate(wanted);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::TimeZone;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::immich::client::ImmichAuth;

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
        let mut sorted = assets.clone();
        sorted.sort();
        for (order, index) in [
            ("desc", 0),
            ("desc", 13),
            ("desc", 29),
            ("asc", 5),
            ("asc", 29),
        ] {
            let mut feed = sorted.clone();
            if order == "desc" {
                feed.reverse();
            }
            let radius = 5;
            let anchor = feed[index].1;
            let expected: Vec<Uuid> = feed
                [index.saturating_sub(radius)..(index + radius + 1).min(feed.len())]
                .iter()
                .map(|(_, id)| *id)
                .collect();
            let mut query = Map::new();
            query.insert("order".into(), order.into());
            let window = around(&immich, query, anchor, radius as u32).await.unwrap();
            let ids: Vec<Uuid> = window.iter().map(|asset| asset.id.source()).collect();
            if ids != expected {
                panic!("{order} window around {index} was {ids:?}, expected {expected:?}");
            }
        }
        let missing = around(&immich, Map::new(), Uuid::new_v4(), 5)
            .await
            .unwrap();
        if !missing.is_empty() {
            panic!("an anchor outside the feed should give an empty window");
        }
    }
}
