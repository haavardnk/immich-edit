use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::Searcher;
use crate::services::search::added::parse_at;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::ids::Ids;
use crate::services::search::months::{Span, within};
use crate::services::search::query::{Direction, MAX_SIZE, PageQuery, Sort, SortField};
use crate::services::single_flight::SingleFlight;
use crate::services::ttl_cache::TtlCache;

const MAX_AHEAD_OF_UTC: TimeDelta = TimeDelta::hours(14);
const MAX_BEHIND_UTC: TimeDelta = TimeDelta::hours(12);
const CACHE_CAP: NonZeroUsize = NonZeroUsize::new(64).unwrap();
const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LocalSpan {
    pub from: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
}

impl LocalSpan {
    fn holds(self, local: NaiveDateTime) -> bool {
        self.from.is_none_or(|from| local >= from) && self.until.is_none_or(|until| local < until)
    }

    fn is_empty(self) -> bool {
        matches!((self.from, self.until), (Some(from), Some(until)) if from >= until)
    }

    fn earliest(self) -> Option<DateTime<Utc>> {
        self.from.map(|from| from.and_utc() - MAX_AHEAD_OF_UTC)
    }

    fn latest(self) -> Option<DateTime<Utc>> {
        self.until.map(|until| until.and_utc() + MAX_BEHIND_UTC)
    }

    fn plan(self) -> Plan {
        let start = self.from.map(|from| from.and_utc() + MAX_BEHIND_UTC);
        let end = self.until.map(|until| until.and_utc() - MAX_AHEAD_OF_UTC);
        if let (Some(start), Some(end)) = (start, end)
            && start >= end
        {
            let joined = self.earliest().zip(self.latest());
            return Plan {
                sure: None,
                slices: joined.into_iter().collect(),
            };
        }
        let lower = self.earliest().zip(start);
        let upper = end.zip(self.latest());
        Plan {
            sure: Some((start, end)),
            slices: lower.into_iter().chain(upper).collect(),
        }
    }
}

type OpenSpan = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

struct Plan {
    sure: Option<OpenSpan>,
    slices: Vec<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Seen {
    id: Uuid,
    at: DateTime<Utc>,
    local: NaiveDateTime,
    added: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
}

impl Seen {
    fn of(asset: &AssetSummary) -> Option<Self> {
        let at = parse_at(asset.file_created_at.as_deref()?)?;
        let local = asset
            .local_date_time
            .as_deref()
            .and_then(parse_at)
            .unwrap_or(at)
            .naive_utc();
        Some(Self {
            id: asset.id.source(),
            at,
            local,
            added: asset.created_at.as_deref().and_then(parse_at),
            updated: asset.updated_at.as_deref().and_then(parse_at),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBounds {
    after: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
    excluded: Vec<Seen>,
}

impl LocalBounds {
    pub fn narrow(&self, filter: &BrowseFilter) -> BrowseFilter {
        BrowseFilter {
            taken_local: None,
            taken_after: filter.taken_after.max(self.after),
            taken_until: earlier(filter.taken_until, self.until),
            ..filter.clone()
        }
    }

    pub fn excluded_by(&self, narrowed: &BrowseFilter) -> HashSet<Uuid> {
        self.excluded
            .iter()
            .filter(|asset| narrowed.takes(asset.at, asset.added, asset.updated))
            .map(|asset| asset.id)
            .collect()
    }

    fn settle(span: LocalSpan, plan: &Plan, seen: &[Seen]) -> Option<Self> {
        let held = || {
            seen.iter()
                .filter(|asset| span.holds(asset.local))
                .map(|asset| asset.at)
        };
        let first = held().min();
        let last = held().max().map(|at| at + TimeDelta::milliseconds(1));
        let (after, until) = match plan.sure {
            Some((start, end)) => (
                start.map(|start| first.map_or(start, |at| at.min(start))),
                end.map(|end| last.map_or(end, |at| at.max(end))),
            ),
            None => (Some(first?), Some(last?)),
        };
        let admits = |at: DateTime<Utc>| {
            after.is_none_or(|after| at >= after) && until.is_none_or(|until| at < until)
        };
        let excluded = seen
            .iter()
            .filter(|asset| !span.holds(asset.local) && admits(asset.at))
            .copied()
            .collect();
        Some(Self {
            after,
            until,
            excluded,
        })
    }
}

type Settled = Option<Arc<LocalBounds>>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct BoundsKey {
    owner: Uuid,
    server_epoch: i64,
    generation: u64,
    basis: String,
    span: LocalSpan,
}

#[derive(Clone)]
pub struct LocalBoundsCache {
    stored: TtlCache<BoundsKey, Settled>,
    flights: SingleFlight<BoundsKey, Result<Settled, AppError>>,
    generation: Arc<AtomicU64>,
}

impl Default for LocalBoundsCache {
    fn default() -> Self {
        Self {
            stored: TtlCache::new(CACHE_CAP, CACHE_TTL),
            flights: SingleFlight::default(),
            generation: Arc::default(),
        }
    }
}

impl LocalBoundsCache {
    pub(super) fn membership_changed(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }
}

pub(super) async fn bounds(
    searcher: &Searcher,
    filter: &BrowseFilter,
    ids: &Ids,
    span: LocalSpan,
) -> Result<Settled, AppError> {
    if span.is_empty() {
        return Ok(None);
    }
    let basis = BrowseFilter {
        taken_local: None,
        taken_after: None,
        taken_before: None,
        taken_until: None,
        added_between: None,
        updated_after: None,
        ..filter.clone()
    };
    let cache = &searcher.caches.local_taken;
    let key = BoundsKey {
        owner: searcher.owner,
        server_epoch: searcher.server_epoch,
        generation: cache.generation.load(Ordering::SeqCst),
        basis: format!("{basis:?}"),
        span,
    };
    if let Some(hit) = cache.stored.get(&key).await {
        return Ok(hit);
    }
    let scope = Scope {
        searcher: searcher.clone(),
        basis,
        ids: ids.clone(),
    };
    let work = || resolve(scope, span, cache.stored.clone(), key.clone());
    cache
        .flights
        .run(key.clone(), work)
        .await
        .ok_or(AppError::Internal)?
}

struct Scope {
    searcher: Searcher,
    basis: BrowseFilter,
    ids: Ids,
}

async fn resolve(
    scope: Scope,
    span: LocalSpan,
    stored: TtlCache<BoundsKey, Settled>,
    key: BoundsKey,
) -> Result<Settled, AppError> {
    let plan = span.plan();
    let mut seen = Vec::new();
    for slice in &plan.slices {
        seen.extend(scope.scan(*slice).await?);
    }
    let settled = LocalBounds::settle(span, &plan, &seen).map(Arc::new);
    stored.put(key, settled.clone()).await;
    Ok(settled)
}

impl Scope {
    async fn scan(&self, slice: Span) -> Result<Vec<Seen>, AppError> {
        let mut query = PageQuery {
            filter: within(&self.basis, slice),
            sort: Sort {
                field: SortField::Taken,
                direction: Direction::Asc,
            },
            size: MAX_SIZE,
            cursor: None,
            with_exif: false,
        };
        let mut seen = Vec::new();
        loop {
            let page = self.searcher.immich_page(&query, &self.ids).await?;
            seen.extend(page.items.iter().filter_map(Seen::of));
            let Some(cursor) = page.next_cursor else {
                return Ok(seen);
            };
            query.cursor = Some(cursor);
        }
    }
}

fn earlier(a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

#[cfg(test)]
mod tests;
