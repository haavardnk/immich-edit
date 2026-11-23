#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::services::search::Searcher;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::months::{Month, Span, within};
use crate::services::search::query::{Direction, MAX_SIZE, PageQuery, Sort, SortField};

const BRACKET_PHOTOS: u64 = 250;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocateQuery {
    #[serde(default)]
    pub filter: BrowseFilter,
    #[serde(default)]
    pub direction: Direction,
    pub month: String,
    pub offset: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Located {
    pub asset_id: Option<AssetKey>,
}

struct Bracket {
    start: i64,
    end: i64,
    ahead_start: u64,
    ahead_end: u64,
}

impl Bracket {
    fn photos(&self) -> u64 {
        self.ahead_start.abs_diff(self.ahead_end)
    }

    fn before_it(&self) -> u64 {
        self.ahead_start.min(self.ahead_end)
    }

    fn splittable(&self) -> bool {
        self.photos() > BRACKET_PHOTOS && self.end - self.start > 1
    }
}

struct MonthWalk<'a> {
    searcher: &'a Searcher,
    filter: &'a BrowseFilter,
    direction: Direction,
    span: Span,
}

impl MonthWalk<'_> {
    async fn ahead(&self, at: DateTime<Utc>) -> Result<u64, AppError> {
        let (start, end) = self.span;
        let range = match self.direction {
            Direction::Desc => (at, end),
            Direction::Asc => (start, at),
        };
        self.searcher.count(&within(self.filter, range)).await
    }

    async fn narrow(&self, offset: u64, total: u64) -> Result<Bracket, AppError> {
        let (start, end) = self.span;
        let (ahead_start, ahead_end) = match self.direction {
            Direction::Desc => (total, 0),
            Direction::Asc => (0, total),
        };
        let mut bracket = Bracket {
            start: start.timestamp_millis(),
            end: end.timestamp_millis(),
            ahead_start,
            ahead_end,
        };
        while bracket.splittable() {
            let middle = bracket.start + (bracket.end - bracket.start) / 2;
            let ahead = self.ahead(instant(middle)?).await?;
            let past_target = ahead > offset;
            let middle_is_start = match self.direction {
                Direction::Desc => past_target,
                Direction::Asc => !past_target,
            };
            if middle_is_start {
                bracket.start = middle;
                bracket.ahead_start = ahead;
            } else {
                bracket.end = middle;
                bracket.ahead_end = ahead;
            }
        }
        Ok(bracket)
    }

    async fn nth_in(&self, bracket: &Bracket, index: u64) -> Result<Option<AssetKey>, AppError> {
        let range = (instant(bracket.start)?, instant(bracket.end)?);
        let mut query = PageQuery {
            filter: within(self.filter, range),
            sort: Sort {
                field: SortField::Taken,
                direction: self.direction,
            },
            size: u32::try_from(index + 1).unwrap_or(MAX_SIZE).min(MAX_SIZE),
            cursor: None,
            with_exif: false,
        };
        let mut skipped = 0;
        let mut last = None;
        loop {
            let page = self.searcher.metadata_page(&query).await?;
            let count = page.items.len() as u64;
            if index < skipped + count {
                let at = usize::try_from(index - skipped).map_err(|_| AppError::Internal)?;
                return Ok(page.items.into_iter().nth(at).map(|asset| asset.id));
            }
            skipped += count;
            last = page.items.into_iter().last().map(|asset| asset.id).or(last);
            match page.next_cursor {
                Some(cursor) if count > 0 => query.cursor = Some(cursor),
                _ => return Ok(last),
            }
        }
    }
}

pub async fn photo_at(searcher: &Searcher, query: &LocateQuery) -> Result<Located, AppError> {
    let month = Month::parse(&query.month)
        .ok_or_else(|| AppError::BadRequest("month must be YYYY-MM".into()))?;
    let walk = MonthWalk {
        searcher,
        filter: &query.filter,
        direction: query.direction,
        span: month.span()?,
    };
    let total = searcher.count(&within(walk.filter, walk.span)).await?;
    let Some(last) = total.checked_sub(1) else {
        return Ok(Located { asset_id: None });
    };
    let offset = query.offset.min(last);
    let bracket = walk.narrow(offset, total).await?;
    let asset_id = walk.nth_in(&bracket, offset - bracket.before_it()).await?;
    Ok(Located { asset_id })
}

fn instant(millis: i64) -> Result<DateTime<Utc>, AppError> {
    DateTime::from_timestamp_millis(millis).ok_or(AppError::Internal)
}
