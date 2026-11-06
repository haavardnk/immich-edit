mod calendar;

use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::{StreamExt, TryStreamExt, stream};
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::services::search::Searcher;
use crate::services::search::added::parse_at;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{Direction, PageQuery, Sort, SortField};
use crate::services::single_flight::SingleFlight;
use crate::services::ttl_cache::TtlCache;
use calendar::{Month, Span, year_span};

const CACHE_CAP: NonZeroUsize = NonZeroUsize::new(64).unwrap();
const CACHE_TTL: Duration = Duration::from_secs(300);
const CONCURRENCY: usize = 8;

#[derive(Clone, PartialEq, Eq, Hash)]
struct MonthsKey {
    owner: Uuid,
    server_epoch: i64,
    query: String,
}

#[derive(Clone)]
struct Tally {
    total: u64,
    months: Vec<MonthCount>,
}

#[derive(Clone)]
pub struct MonthCountCache {
    stored: TtlCache<MonthsKey, Tally>,
    flights: SingleFlight<MonthsKey, Result<Tally, AppError>>,
}

impl Default for MonthCountCache {
    fn default() -> Self {
        Self {
            stored: TtlCache::new(CACHE_CAP, CACHE_TTL),
            flights: SingleFlight::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthCount {
    pub month: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct MonthCounts {
    pub months: Vec<MonthCount>,
}

pub async fn count(searcher: &Searcher, filter: &BrowseFilter) -> Result<MonthCounts, AppError> {
    let Some(body) = searcher.count_body(filter).await? else {
        return Ok(MonthCounts { months: Vec::new() });
    };
    let key = MonthsKey {
        owner: searcher.owner,
        server_epoch: searcher.server_epoch,
        query: body.to_string(),
    };
    let cache = &searcher.caches.months;
    if let Some(hit) = cache.stored.get(&key).await
        && searcher.count(filter).await? == hit.total
    {
        return Ok(MonthCounts { months: hit.months });
    }
    let work = || {
        refresh(
            searcher.clone(),
            filter.clone(),
            cache.stored.clone(),
            key.clone(),
        )
    };
    let tally = cache
        .flights
        .run(key.clone(), work)
        .await
        .ok_or(AppError::Internal)??;
    Ok(MonthCounts {
        months: tally.months,
    })
}

async fn refresh(
    searcher: Searcher,
    filter: BrowseFilter,
    stored: TtlCache<MonthsKey, Tally>,
    key: MonthsKey,
) -> Result<Tally, AppError> {
    let total = searcher.count(&filter).await?;
    let months = match total {
        0 => Vec::new(),
        _ => tally(&searcher, &filter).await?,
    };
    let fresh = Tally { total, months };
    stored.put(key, fresh.clone()).await;
    Ok(fresh)
}

async fn tally(searcher: &Searcher, filter: &BrowseFilter) -> Result<Vec<MonthCount>, AppError> {
    let (newest, oldest) = tokio::try_join!(
        edge(searcher, filter, Direction::Desc),
        edge(searcher, filter, Direction::Asc)
    )?;
    let (Some(newest), Some(oldest)) = (newest, oldest) else {
        return Ok(Vec::new());
    };
    let first = Month::of(oldest);
    let last = Month::of(newest);
    let occupied = occupied_years(searcher, filter, first.year(), last.year()).await?;
    let months: Vec<Month> = first
        .through(last)
        .filter(|month| occupied.contains(&month.year()))
        .collect();
    let spans = months
        .iter()
        .map(|month| month.span())
        .collect::<Result<Vec<Span>, AppError>>()?;
    let counts = counts(searcher, filter, spans).await?;
    Ok(months
        .into_iter()
        .zip(counts)
        .rev()
        .filter(|(_, count)| *count > 0)
        .map(|(month, count)| MonthCount {
            month: month.label(),
            count,
        })
        .collect())
}

async fn occupied_years(
    searcher: &Searcher,
    filter: &BrowseFilter,
    first: i32,
    last: i32,
) -> Result<HashSet<i32>, AppError> {
    let middle: Vec<i32> = (first + 1..last).collect();
    let spans = middle
        .iter()
        .map(|year| year_span(*year))
        .collect::<Result<Vec<Span>, AppError>>()?;
    let counts = counts(searcher, filter, spans).await?;
    Ok(middle
        .into_iter()
        .zip(counts)
        .filter(|(_, count)| *count > 0)
        .map(|(year, _)| year)
        .chain([first, last])
        .collect())
}

async fn edge(
    searcher: &Searcher,
    filter: &BrowseFilter,
    direction: Direction,
) -> Result<Option<DateTime<Utc>>, AppError> {
    let query = PageQuery {
        filter: filter.clone(),
        sort: Sort {
            field: SortField::Taken,
            direction,
        },
        size: 1,
        cursor: None,
        with_exif: false,
    };
    let page = searcher.metadata_page(&query).await?;
    let Some(asset) = page.items.into_iter().next() else {
        return Ok(None);
    };
    let raw = asset.file_created_at.unwrap_or_default();
    let taken = parse_at(&raw)
        .ok_or_else(|| ImmichError::Decode(format!("search fileCreatedAt {raw:?}")))?;
    Ok(Some(taken))
}

async fn counts(
    searcher: &Searcher,
    filter: &BrowseFilter,
    spans: Vec<Span>,
) -> Result<Vec<u64>, AppError> {
    stream::iter(spans)
        .map(|span| {
            let bounded = within(filter, span);
            async move { searcher.count(&bounded).await }
        })
        .buffered(CONCURRENCY)
        .try_collect()
        .await
}

fn within(filter: &BrowseFilter, (start, end): Span) -> BrowseFilter {
    BrowseFilter {
        taken_after: Some(filter.taken_after.map_or(start, |after| after.max(start))),
        taken_until: Some(end),
        ..filter.clone()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone};

    use super::*;

    #[test]
    fn a_bounded_filter_keeps_the_later_start_and_the_picked_end() {
        let march = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
        let april = Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap();
        let picked_after = Utc.with_ymd_and_hms(2024, 3, 10, 0, 0, 0).unwrap();
        let picked_before = Utc.with_ymd_and_hms(2024, 3, 20, 0, 0, 0).unwrap();
        let cases = [
            (None, None, march),
            (Some(picked_after), Some(picked_before), picked_after),
            (Some(march - TimeDelta::days(9)), None, march),
        ];
        for (taken_after, taken_before, start) in cases {
            let filter = BrowseFilter {
                taken_after,
                taken_before,
                favorite: true,
                ..BrowseFilter::default()
            };
            let bounded = within(&filter, (march, april));
            let kept = (
                bounded.taken_after,
                bounded.taken_before,
                bounded.taken_until,
                bounded.favorite,
            );
            if kept != (Some(start), taken_before, Some(april), true) {
                panic!("bounded filter kept {kept:?}");
            }
        }
    }
}
