use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use chrono::{DateTime, Months, NaiveDate, NaiveTime, TimeDelta, Utc};
use futures_util::future::try_join_all;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::Searcher;
use crate::services::search::added::buckets::{self, Entry};
use crate::services::search::added::hydrate::hydrate;
use crate::services::search::added::sources::{self, Params};
use crate::services::search::cursor::Cursor;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::ids::Ids;
use crate::services::search::query::Direction;

pub(super) const SCAN_CHUNKS: usize = 16;
const CONCURRENCY: usize = 4;

struct Source {
    params: Params,
    counts: HashMap<String, u64>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Position {
    bucket: usize,
    offset: usize,
}

pub(super) struct Scan {
    pub items: Vec<AssetSummary>,
    pub resume: Option<Cursor>,
}

pub(super) struct Timeline<'a> {
    searcher: &'a Searcher,
    sources: Arc<[Source]>,
    buckets: Vec<String>,
    order: Direction,
    direction: Direction,
}

impl<'a> Timeline<'a> {
    pub async fn load(
        searcher: &'a Searcher,
        filter: &BrowseFilter,
        ids: &Ids,
        direction: Direction,
    ) -> Result<Self, AppError> {
        let plans = sources::sources(searcher, filter, ids, direction).await?;
        let lists = try_join_all(
            plans
                .iter()
                .map(|params| searcher.immich.timeline_buckets(params)),
        )
        .await?;
        let (after, before) = filter.added_bounds();
        let mut months = BTreeSet::new();
        let mut sources = Vec::new();
        for (params, list) in plans.into_iter().zip(lists) {
            let counts: HashMap<String, u64> = list
                .into_iter()
                .filter(|bucket| overlaps(&bucket.time_bucket, after, before))
                .map(|bucket| (bucket.time_bucket, bucket.count))
                .collect();
            months.extend(counts.keys().cloned());
            sources.push(Source { params, counts });
        }
        let mut buckets: Vec<String> = months.into_iter().collect();
        if direction == Direction::Desc {
            buckets.reverse();
        }
        Ok(Self {
            searcher,
            sources: sources.into(),
            buckets,
            order: direction,
            direction,
        })
    }

    pub fn reversed(&self) -> Self {
        Self {
            searcher: self.searcher,
            sources: self.sources.clone(),
            buckets: self.buckets.iter().rev().cloned().collect(),
            order: self.order,
            direction: match self.direction {
                Direction::Asc => Direction::Desc,
                Direction::Desc => Direction::Asc,
            },
        }
    }

    async fn entries(&self, index: usize) -> Result<Arc<[Entry]>, AppError> {
        let Some(bucket) = self.buckets.get(index) else {
            return Ok(Arc::from([]));
        };
        let lists = try_join_all(self.sources.iter().filter_map(|source| {
            let count = *source.counts.get(bucket)?;
            Some(buckets::entries(
                self.searcher,
                &source.params,
                bucket,
                count,
            ))
        }))
        .await?;
        if let [only] = lists.as_slice()
            && self.direction == self.order
        {
            return Ok(only.clone());
        }
        let mut merged: Vec<Entry> = lists.iter().flat_map(|list| list.iter().copied()).collect();
        merged.sort_by(|a, b| rank(self.order, a.at, b.at));
        if self.direction != self.order {
            merged.reverse();
        }
        Ok(merged.into())
    }

    pub async fn resume(
        &self,
        bucket: &str,
        at: DateTime<Utc>,
        id: Uuid,
    ) -> Result<Position, AppError> {
        let Some(index) = self
            .buckets
            .iter()
            .position(|month| rank(self.direction, month.as_str(), bucket) != Ordering::Less)
        else {
            return Ok(Position {
                bucket: self.buckets.len(),
                offset: 0,
            });
        };
        if self.buckets[index] != bucket {
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
                .position(|entry| rank(self.direction, entry.at, at) == Ordering::Greater)
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
            bucket: self.buckets[bucket].clone(),
            at: entry.at,
            id: entry.id,
        }
    }

    pub async fn scan(
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

fn rank<T: Ord>(direction: Direction, a: T, b: T) -> Ordering {
    match direction {
        Direction::Asc => a.cmp(&b),
        Direction::Desc => b.cmp(&a),
    }
}

fn overlaps(bucket: &str, after: Option<DateTime<Utc>>, before: Option<DateTime<Utc>>) -> bool {
    let Some(start) = bucket
        .get(..10)
        .and_then(|day| NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
    else {
        return true;
    };
    let end = start + Months::new(1);
    let margin = TimeDelta::days(1);
    let starts_before_end =
        before.is_none_or(|before| start.and_time(NaiveTime::MIN).and_utc() <= before + margin);
    let ends_after_start =
        after.is_none_or(|after| end.and_time(NaiveTime::MIN).and_utc() >= after - margin);
    starts_before_end && ends_after_start
}
