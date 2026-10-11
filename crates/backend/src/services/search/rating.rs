use std::collections::VecDeque;

use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::Searcher;
use crate::services::search::filter::{BrowseFilter, RatingMatch};
use crate::services::search::query::{Direction, MAX_SIZE, PageQuery, Sort, SortField};

const RATINGS: [i8; 7] = [-1, 0, 1, 2, 3, 4, 5];

fn admits(filter: Option<RatingMatch>, group: Option<i8>) -> bool {
    match filter {
        None => true,
        Some(RatingMatch::Eq(stars)) => group == Some(stars),
        Some(RatingMatch::Gte(stars)) => group.is_some_and(|rating| rating >= stars),
        Some(RatingMatch::Lte(stars)) => group.is_none_or(|rating| rating <= stars),
        Some(RatingMatch::Unrated) => group.is_none(),
    }
}

fn groups(filter: Option<RatingMatch>, direction: Direction) -> Vec<Option<i8>> {
    let rated: Vec<Option<i8>> = match direction {
        Direction::Asc => RATINGS.iter().copied().map(Some).collect(),
        Direction::Desc => RATINGS.iter().rev().copied().map(Some).collect(),
    };
    rated
        .into_iter()
        .chain(std::iter::once(None))
        .filter(|group| admits(filter, *group))
        .collect()
}

fn group_query(
    filter: &BrowseFilter,
    group: Option<i8>,
    direction: Direction,
    size: u32,
) -> PageQuery {
    PageQuery {
        filter: BrowseFilter {
            rating: Some(group.map_or(RatingMatch::Unrated, RatingMatch::Eq)),
            ..filter.clone()
        },
        sort: Sort {
            field: SortField::Rating,
            direction,
        },
        size,
        cursor: None,
        with_exif: true,
    }
}

pub async fn around(
    searcher: &Searcher,
    filter: &BrowseFilter,
    direction: Direction,
    anchor: AssetSummary,
    radius: u32,
) -> Result<Vec<AssetSummary>, AppError> {
    let rating = anchor.exif_info.as_ref().and_then(|exif| exif.rating);
    let order = groups(filter.rating, direction);
    let Some(at) = order
        .iter()
        .position(|group| group.map(i32::from) == rating)
    else {
        return Ok(vec![anchor]);
    };
    let wanted = radius as usize;
    let query = group_query(filter, order[at], direction, MAX_SIZE);
    let Some((mut before, mut after)) = locate(searcher, query, anchor.id.source(), wanted).await?
    else {
        return Ok(vec![anchor]);
    };
    let reverse = match direction {
        Direction::Asc => Direction::Desc,
        Direction::Desc => Direction::Asc,
    };
    for group in order[..at].iter().rev() {
        let need = wanted - before.len();
        if need == 0 {
            break;
        }
        let query = group_query(filter, *group, reverse, need as u32);
        before.extend(take(searcher, query, need).await?);
    }
    for group in &order[at + 1..] {
        let need = wanted - after.len();
        if need == 0 {
            break;
        }
        let query = group_query(filter, *group, direction, need as u32);
        after.extend(take(searcher, query, need).await?);
    }
    Ok(before
        .into_iter()
        .rev()
        .chain(std::iter::once(anchor))
        .chain(after)
        .collect())
}

async fn locate(
    searcher: &Searcher,
    mut query: PageQuery,
    anchor: Uuid,
    wanted: usize,
) -> Result<Option<(Vec<AssetSummary>, Vec<AssetSummary>)>, AppError> {
    let mut before: VecDeque<AssetSummary> = VecDeque::new();
    let mut after: Option<Vec<AssetSummary>> = None;
    loop {
        let page = searcher.page(&query).await?;
        for asset in page.items {
            match after.as_mut() {
                Some(after) => after.push(asset),
                None if asset.id.source() == anchor => after = Some(Vec::new()),
                None => {
                    before.push_back(asset);
                    if before.len() > wanted {
                        before.pop_front();
                    }
                }
            }
        }
        if after.as_ref().is_some_and(|after| after.len() >= wanted) {
            break;
        }
        let Some(cursor) = page.next_cursor else {
            break;
        };
        query.cursor = Some(cursor);
    }
    Ok(after.map(|mut after| {
        after.truncate(wanted);
        (before.into_iter().rev().collect(), after)
    }))
}

async fn take(
    searcher: &Searcher,
    mut query: PageQuery,
    wanted: usize,
) -> Result<Vec<AssetSummary>, AppError> {
    let mut out = Vec::new();
    loop {
        let page = searcher.page(&query).await?;
        out.extend(page.items);
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
    use std::cmp::Ordering;
    use std::time::Duration;

    use serde_json::{Value, json};
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::immich::ImmichClient;
    use crate::immich::client::ImmichAuth;
    use crate::services::search::SearchShape;
    use crate::services::search::window;

    type Rated = (Option<i8>, Uuid);

    fn feed_order(a: &Rated, b: &Rated, direction: Direction) -> Ordering {
        let by_rating = match (a.0, b.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(x), Some(y)) => x.cmp(&y),
        };
        let ordering = by_rating.then(a.1.cmp(&b.1));
        match direction {
            Direction::Asc => ordering,
            Direction::Desc => ordering.reverse(),
        }
    }

    struct FakeStructuredSearch {
        assets: Vec<Rated>,
    }

    impl Respond for FakeStructuredSearch {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let body: Value = request.body_json().unwrap();
            let filter = &body["filter"];
            let id = filter["id"]["eq"].as_str();
            let rating = filter["rating"].get("eq");
            let direction = match body["orderBy"]["direction"].as_str() {
                Some("asc") => Direction::Asc,
                _ => Direction::Desc,
            };
            let mut matched: Vec<Rated> = self
                .assets
                .iter()
                .filter(|(_, asset)| id.is_none_or(|i| asset.to_string() == i))
                .filter(|(stars, _)| rating.is_none_or(|eq| eq.as_i64() == stars.map(i64::from)))
                .copied()
                .collect();
            matched.sort_by(|a, b| feed_order(a, b, direction));
            let size = (body["size"].as_u64().unwrap() as usize).min(7);
            let start: usize = body["cursor"].as_str().map_or(0, |c| c.parse().unwrap());
            let items: Vec<Value> = matched
                .iter()
                .skip(start)
                .take(size)
                .map(|(stars, asset)| {
                    json!({ "id": asset, "type": "IMAGE", "exifInfo": { "rating": stars } })
                })
                .collect();
            let next = (start + size < matched.len()).then(|| (start + size).to_string());
            ResponseTemplate::new(200).set_body_json(json!({
                "assets": { "items": items, "count": items.len(), "total": matched.len(), "nextCursor": next },
            }))
        }
    }

    #[tokio::test]
    async fn rating_window_matches_the_full_feed_around_the_anchor() {
        let stars = [None, Some(-1), Some(1), Some(3), Some(3), Some(5)];
        let assets: Vec<Rated> = (0..60)
            .map(|i| (stars[i % stars.len()], Uuid::new_v4()))
            .collect();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/search/metadata"))
            .respond_with(FakeStructuredSearch {
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
        let searcher = Searcher::new(immich, SearchShape::Structured);
        let cases = [
            (None, Direction::Desc, 0_usize),
            (None, Direction::Desc, 25),
            (None, Direction::Desc, 59),
            (None, Direction::Asc, 12),
            (None, Direction::Asc, 45),
            (Some(RatingMatch::Lte(3)), Direction::Desc, 20),
            (Some(RatingMatch::Gte(3)), Direction::Asc, 15),
        ];
        for (rating, direction, index) in cases {
            let filter = BrowseFilter {
                rating,
                ..BrowseFilter::default()
            };
            let mut feed: Vec<Rated> = assets
                .iter()
                .filter(|(stars, _)| admits(rating, *stars))
                .copied()
                .collect();
            feed.sort_by(|a, b| feed_order(a, b, direction));
            let radius = 8;
            let expected: Vec<Uuid> = feed
                [index.saturating_sub(radius)..(index + radius + 1).min(feed.len())]
                .iter()
                .map(|(_, id)| *id)
                .collect();
            let sort = Sort {
                field: SortField::Rating,
                direction,
            };
            let found = window::around(&searcher, &filter, sort, feed[index].1, radius as u32)
                .await
                .unwrap();
            let ids: Vec<Uuid> = found.iter().map(|asset| asset.id.source()).collect();
            if ids != expected {
                panic!(
                    "{rating:?} {direction:?} window around {index} was {ids:?}, expected {expected:?}"
                );
            }
        }
    }
}
