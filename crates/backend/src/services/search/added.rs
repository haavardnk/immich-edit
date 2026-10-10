use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, NaiveDateTime, TimeDelta, Utc};
use futures_util::future::try_join_all;
use lru::LruCache;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::dto::{AssetSummary, TimeBucket};
use crate::services::search::cursor::Cursor;
use crate::services::search::filter::{BrowseFilter, Scope, Visibility};
use crate::services::search::query::{Direction, MAX_SIZE, PageQuery, Sort};
use crate::services::search::{SearchPage, Searcher};

const CACHE_CAP: usize = 64;
const CACHE_TTL: Duration = Duration::from_secs(300);
const SCAN_CHUNKS: usize = 16;
const CONCURRENCY: usize = 4;

#[derive(Clone, PartialEq, Eq, Hash)]
struct BucketKey {
    owner: Uuid,
    server_epoch: i64,
    params: String,
    bucket: String,
}

struct CachedBucket {
    fetched: Instant,
    entries: Arc<[Entry]>,
}

#[derive(Clone)]
pub struct TimelineCache {
    inner: Arc<Mutex<LruCache<BucketKey, CachedBucket>>>,
}

impl Default for TimelineCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CAP).unwrap(),
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    at: DateTime<Utc>,
    id: Uuid,
}

#[derive(Debug, Clone, Copy, Default)]
struct Position {
    bucket: usize,
    offset: usize,
}

struct Scan {
    items: Vec<AssetSummary>,
    resume: Option<Cursor>,
}

struct Timeline<'a> {
    searcher: &'a Searcher,
    params: Vec<(&'static str, String)>,
    buckets: Vec<TimeBucket>,
    direction: Direction,
}

pub async fn page(searcher: &Searcher, query: &PageQuery) -> Result<SearchPage, AppError> {
    let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
    if searcher.tags(&query.filter).await?.is_none() {
        return Ok(SearchPage::empty());
    }
    let timeline = Timeline::load(searcher, &query.filter, query.sort.direction).await?;
    let start = match cursor {
        None => Position::default(),
        Some(Cursor::Added { bucket, at, id }) => timeline.resume(&bucket, at, id).await?,
        Some(_) => return Err(Cursor::mismatch()),
    };
    let scan = timeline
        .scan(start, query.size as usize, &query.filter, query.with_exif)
        .await?;
    Ok(SearchPage {
        items: scan.items,
        next_cursor: scan.resume.map(|cursor| cursor.encode()),
    })
}

pub async fn around(
    searcher: &Searcher,
    filter: &BrowseFilter,
    direction: Direction,
    anchor: AssetSummary,
    radius: u32,
) -> Result<Vec<AssetSummary>, AppError> {
    let Some(at) = anchor.created_at.as_deref().and_then(parse_at) else {
        return Ok(vec![anchor]);
    };
    let bucket = format!("{:04}-{:02}-01", at.year(), at.month());
    let id = anchor.id.source();
    let newer = side(searcher, filter, Direction::Asc, (&bucket, at, id), radius).await?;
    let older = side(searcher, filter, Direction::Desc, (&bucket, at, id), radius).await?;
    let (before, after) = match direction {
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
    direction: Direction,
    (bucket, at, id): (&str, DateTime<Utc>, Uuid),
    radius: u32,
) -> Result<Vec<AssetSummary>, AppError> {
    let timeline = Timeline::load(searcher, filter, direction).await?;
    let start = timeline.resume(bucket, at, id).await?;
    Ok(timeline
        .scan(start, radius as usize, filter, true)
        .await?
        .items)
}

fn params(filter: &BrowseFilter, direction: Direction) -> Vec<(&'static str, String)> {
    let order = match direction {
        Direction::Asc => "asc",
        Direction::Desc => "desc",
    };
    let mut params = vec![("orderBy", "createdAt".into()), ("order", order.into())];
    match filter.scope {
        Some(Scope::Album(id)) => params.push(("albumId", id.to_string())),
        Some(Scope::Person(id)) => params.push(("personId", id.to_string())),
        Some(Scope::Tag(id)) => params.push(("tagId", id.to_string())),
        None => {}
    }
    let visibility = filter.visibility.map(|visibility| match visibility {
        Visibility::Timeline => "timeline",
        Visibility::Archive => "archive",
        Visibility::Hidden => "hidden",
    });
    if let Some(visibility) = visibility {
        params.push(("visibility", visibility.into()));
    }
    let album = matches!(filter.scope, Some(Scope::Album(_)));
    if filter.visibility == Some(Visibility::Timeline) && !album {
        params.push(("withPartners", "true".into()));
    }
    params
}

fn parse_at(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|at| at.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|at| at.and_utc())
        })
}

impl<'a> Timeline<'a> {
    async fn load(
        searcher: &'a Searcher,
        filter: &BrowseFilter,
        direction: Direction,
    ) -> Result<Self, AppError> {
        let params = params(filter, direction);
        let mut buckets = searcher.immich.timeline_buckets(&params).await?;
        buckets.sort_by(|a, b| a.time_bucket.cmp(&b.time_bucket));
        if direction == Direction::Desc {
            buckets.reverse();
        }
        Ok(Self {
            searcher,
            params,
            buckets,
            direction,
        })
    }

    fn rank<T: Ord>(&self, a: T, b: T) -> Ordering {
        match self.direction {
            Direction::Asc => a.cmp(&b),
            Direction::Desc => b.cmp(&a),
        }
    }

    async fn entries(&self, index: usize) -> Result<Arc<[Entry]>, AppError> {
        let Some(bucket) = self.buckets.get(index) else {
            return Ok(Arc::from([]));
        };
        let key = BucketKey {
            owner: self.searcher.owner,
            server_epoch: self.searcher.server_epoch,
            params: self
                .params
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("&"),
            bucket: bucket.time_bucket.clone(),
        };
        let cache = &self.searcher.timelines.inner;
        if let Some(hit) = cache.lock().await.get(&key)
            && hit.fetched.elapsed() < CACHE_TTL
            && hit.entries.len() as u64 == bucket.count
        {
            return Ok(hit.entries.clone());
        }
        let assets = self
            .searcher
            .immich
            .timeline_bucket(&self.params, &bucket.time_bucket)
            .await?;
        if assets.id.len() != assets.created_at.len() {
            return Err(
                ImmichError::Decode("timeline bucket columns differ in length".into()).into(),
            );
        }
        let entries = assets
            .id
            .into_iter()
            .zip(assets.created_at)
            .map(|(id, raw)| {
                parse_at(&raw)
                    .map(|at| Entry { at, id })
                    .ok_or_else(|| ImmichError::Decode(format!("timeline createdAt {raw:?}")))
            })
            .collect::<Result<Arc<[Entry]>, ImmichError>>()?;
        cache.lock().await.put(
            key,
            CachedBucket {
                fetched: Instant::now(),
                entries: entries.clone(),
            },
        );
        Ok(entries)
    }

    async fn resume(
        &self,
        bucket: &str,
        at: DateTime<Utc>,
        id: Uuid,
    ) -> Result<Position, AppError> {
        let Some(index) = self
            .buckets
            .iter()
            .position(|b| self.rank(b.time_bucket.as_str(), bucket) != Ordering::Less)
        else {
            return Ok(Position {
                bucket: self.buckets.len(),
                offset: 0,
            });
        };
        if self.buckets[index].time_bucket != bucket {
            return Ok(Position {
                bucket: index,
                offset: 0,
            });
        }
        let entries = self.entries(index).await?;
        let offset = match entries.iter().position(|entry| entry.id == id) {
            Some(found) => found + 1,
            None => entries
                .iter()
                .position(|entry| self.rank(entry.at, at) == Ordering::Greater)
                .unwrap_or(entries.len()),
        };
        Ok(Position {
            bucket: index,
            offset,
        })
    }

    async fn take(
        &self,
        position: &mut Position,
        size: usize,
    ) -> Result<Vec<(usize, Entry)>, AppError> {
        let mut chunk = Vec::new();
        while chunk.len() < size && position.bucket < self.buckets.len() {
            let entries = self.entries(position.bucket).await?;
            let taken = entries
                .iter()
                .skip(position.offset)
                .take(size - chunk.len())
                .map(|entry| (position.bucket, *entry));
            let before = chunk.len();
            chunk.extend(taken);
            position.offset += chunk.len() - before;
            if position.offset >= entries.len() {
                position.bucket += 1;
                position.offset = 0;
            }
        }
        Ok(chunk)
    }

    fn cursor(&self, (bucket, entry): (usize, Entry)) -> Cursor {
        Cursor::Added {
            bucket: self.buckets[bucket].time_bucket.clone(),
            at: entry.at,
            id: entry.id,
        }
    }

    async fn scan(
        &self,
        mut position: Position,
        size: usize,
        filter: &BrowseFilter,
        with_exif: bool,
    ) -> Result<Scan, AppError> {
        let mut items = Vec::new();
        let mut scanned = 0;
        let mut last = None;
        while scanned < SCAN_CHUNKS {
            let width = if scanned == 0 {
                1
            } else {
                CONCURRENCY.min(SCAN_CHUNKS - scanned)
            };
            let mut batch = Vec::new();
            while batch.len() < width {
                let chunk = self.take(&mut position, size).await?;
                if chunk.is_empty() {
                    break;
                }
                batch.push(chunk);
            }
            if batch.is_empty() {
                return Ok(Scan {
                    items,
                    resume: None,
                });
            }
            scanned += batch.len();
            let hydrated = try_join_all(
                batch
                    .iter()
                    .map(|chunk| hydrate(self.searcher, filter, with_exif, chunk)),
            )
            .await?;
            let mut found: HashMap<Uuid, AssetSummary> = hydrated.into_iter().flatten().collect();
            for step in batch.into_iter().flatten() {
                last = Some(step);
                let Some(asset) = found.remove(&step.1.id) else {
                    continue;
                };
                items.push(asset);
                if items.len() == size {
                    return Ok(Scan {
                        items,
                        resume: Some(self.cursor(step)),
                    });
                }
            }
        }
        Ok(Scan {
            items,
            resume: last.map(|step| self.cursor(step)),
        })
    }
}

async fn hydrate(
    searcher: &Searcher,
    filter: &BrowseFilter,
    with_exif: bool,
    chunk: &[(usize, Entry)],
) -> Result<HashMap<Uuid, AssetSummary>, AppError> {
    let times = || chunk.iter().map(|(_, entry)| entry.at);
    let (Some(low), Some(high)) = (times().min(), times().max()) else {
        return Ok(HashMap::new());
    };
    let wanted: HashSet<Uuid> = chunk.iter().map(|(_, entry)| entry.id).collect();
    let mut query = PageQuery {
        filter: BrowseFilter {
            added_between: Some((
                low - TimeDelta::milliseconds(1),
                high + TimeDelta::milliseconds(1),
            )),
            ..filter.clone()
        },
        sort: Sort::default(),
        size: MAX_SIZE,
        cursor: None,
        with_exif,
    };
    let mut found = HashMap::new();
    loop {
        let page = searcher.metadata_page(&query).await?;
        found.extend(
            page.items
                .into_iter()
                .filter(|asset| wanted.contains(&asset.id.source()))
                .map(|asset| (asset.id.source(), asset)),
        );
        let Some(cursor) = page.next_cursor else {
            return Ok(found);
        };
        query.cursor = Some(cursor);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, TimeZone};
    use serde_json::{Value, json};
    use url::Url;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::immich::ImmichClient;
    use crate::immich::client::ImmichAuth;
    use crate::services::search::SearchShape;
    use crate::services::search::query::SortField;
    use crate::services::search::window;

    struct FakeImmich {
        assets: Vec<(DateTime<Utc>, Uuid, bool)>,
    }

    fn month(at: DateTime<Utc>) -> String {
        format!("{:04}-{:02}-01", at.year(), at.month())
    }

    impl FakeImmich {
        fn timeline(&self, request: &Request) -> ResponseTemplate {
            let query: HashMap<String, String> = request.url.query_pairs().into_owned().collect();
            if query.get("orderBy").map(String::as_str) != Some("createdAt") {
                return ResponseTemplate::new(400);
            }
            let mut assets = self.assets.clone();
            assets.sort();
            if query.get("order").map(String::as_str) != Some("asc") {
                assets.reverse();
            }
            let Some(bucket) = query.get("timeBucket") else {
                let mut months: Vec<(String, u64)> = Vec::new();
                for (at, _, _) in &assets {
                    match months.last_mut() {
                        Some((key, count)) if *key == month(*at) => *count += 1,
                        _ => months.push((month(*at), 1)),
                    }
                }
                let body: Vec<Value> = months
                    .into_iter()
                    .map(|(key, count)| json!({ "timeBucket": key, "count": count }))
                    .collect();
                return ResponseTemplate::new(200).set_body_json(body);
            };
            let (created, ids): (Vec<String>, Vec<Uuid>) = assets
                .iter()
                .filter(|(at, _, _)| month(*at) == *bucket)
                .map(|(at, id, _)| {
                    (
                        at.naive_utc().format("%Y-%m-%dT%H:%M:%S%.6f").to_string(),
                        *id,
                    )
                })
                .unzip();
            ResponseTemplate::new(200).set_body_json(json!({ "id": ids, "createdAt": created }))
        }

        fn search(&self, request: &Request) -> ResponseTemplate {
            let body: Value = request.body_json().unwrap();
            let bound = |name: &str| {
                body.get(name)
                    .and_then(Value::as_str)
                    .map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
            };
            let after = bound("createdAfter");
            let before = bound("createdBefore");
            let favorite = body.get("isFavorite").and_then(Value::as_bool);
            let id = body.get("id").and_then(Value::as_str);
            let items: Vec<Value> = self
                .assets
                .iter()
                .filter(|(at, _, _)| after.is_none_or(|a| *at >= a))
                .filter(|(at, _, _)| before.is_none_or(|b| *at <= b))
                .filter(|(_, _, fav)| favorite.is_none_or(|f| *fav == f))
                .filter(|(_, asset, _)| id.is_none_or(|i| asset.to_string() == i))
                .map(|(at, asset, fav)| {
                    json!({
                        "id": asset,
                        "type": "IMAGE",
                        "isFavorite": fav,
                        "createdAt": at.to_rfc3339_opts(SecondsFormat::Micros, true),
                    })
                })
                .collect();
            ResponseTemplate::new(200).set_body_json(json!({
                "assets": { "items": items, "count": items.len(), "total": items.len(), "nextPage": null },
            }))
        }
    }

    impl Respond for FakeImmich {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            match request.url.path() {
                "/api/search/metadata" => self.search(request),
                _ => self.timeline(request),
            }
        }
    }

    type Favorite = fn(usize) -> bool;

    async fn searcher(
        favorite: Favorite,
    ) -> (MockServer, Searcher, Vec<(DateTime<Utc>, Uuid, bool)>) {
        let base = Utc.with_ymd_and_hms(2024, 1, 20, 8, 0, 0).unwrap();
        let assets: Vec<(DateTime<Utc>, Uuid, bool)> = (0..40)
            .map(|i| {
                let at = base
                    + TimeDelta::days(2 * i as i64)
                    + TimeDelta::microseconds(1_234_567 * i as i64);
                (at, Uuid::new_v4(), favorite(i))
            })
            .collect();
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(FakeImmich {
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
        (server, Searcher::new(immich, SearchShape::Flat), assets)
    }

    async fn feed(searcher: &Searcher, direction: Direction, size: u32) -> (Vec<Uuid>, usize) {
        let mut query = PageQuery {
            filter: BrowseFilter {
                favorite: true,
                ..BrowseFilter::default()
            },
            sort: Sort {
                field: SortField::Added,
                direction,
            },
            size,
            cursor: None,
            with_exif: false,
        };
        let mut ids = Vec::new();
        let mut empty = 0;
        loop {
            let page = searcher.page(&query).await.unwrap();
            if page.items.is_empty() && page.next_cursor.is_some() {
                empty += 1;
            }
            ids.extend(page.items.iter().map(|asset| asset.id.source()));
            let Some(cursor) = page.next_cursor else {
                return (ids, empty);
            };
            query.cursor = Some(cursor);
        }
    }

    #[tokio::test]
    async fn the_added_feed_walks_the_timeline_through_the_filter() {
        let cases: [(Favorite, u32, bool); 3] = [
            (|i| i % 3 == 0, 3, false),
            (|i| i % 3 == 0, 1000, false),
            (|i| i == 0, 1, true),
        ];
        for (favorite, size, sparse) in cases {
            let (_server, searcher, assets) = searcher(favorite).await;
            let mut expected: Vec<(DateTime<Utc>, Uuid)> = assets
                .iter()
                .filter(|(_, _, fav)| *fav)
                .map(|(at, id, _)| (*at, *id))
                .collect();
            expected.sort();
            let oldest_first: Vec<Uuid> = expected.iter().map(|(_, id)| *id).collect();
            let newest_first: Vec<Uuid> = oldest_first.iter().rev().copied().collect();
            for (direction, wanted) in [
                (Direction::Asc, &oldest_first),
                (Direction::Desc, &newest_first),
            ] {
                let (ids, empty) = feed(&searcher, direction, size).await;
                if ids != *wanted {
                    panic!("{direction:?} size {size} feed was {ids:?}, expected {wanted:?}");
                }
                if sparse && direction == Direction::Desc && empty == 0 {
                    panic!("a sparse filter should hand back empty pages that keep a cursor");
                }
            }
        }
    }

    #[tokio::test]
    async fn the_added_window_matches_the_feed_around_the_anchor() {
        let (_server, searcher, _) = searcher(|i| i % 2 == 0).await;
        let filter = BrowseFilter {
            favorite: true,
            ..BrowseFilter::default()
        };
        for direction in [Direction::Desc, Direction::Asc] {
            let (feed, _) = feed(&searcher, direction, 50).await;
            let sort = Sort {
                field: SortField::Added,
                direction,
            };
            for index in [0, 7, feed.len() - 1] {
                let window = window::around(&searcher, &filter, sort, feed[index], 3)
                    .await
                    .unwrap();
                let ids: Vec<Uuid> = window.iter().map(|asset| asset.id.source()).collect();
                let expected = &feed[index.saturating_sub(3)..(index + 4).min(feed.len())];
                if ids != expected {
                    panic!(
                        "{direction:?} window around {index} was {ids:?}, expected {expected:?}"
                    );
                }
            }
        }
    }
}
