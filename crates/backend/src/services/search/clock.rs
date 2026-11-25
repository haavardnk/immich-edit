use chrono::{DateTime, TimeDelta, Utc};

use crate::immich::dto::AssetSummary;
use crate::services::search::added::parse_at;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::local_taken::LocalSpan;

pub const MAX_AHEAD_OF_UTC: TimeDelta = TimeDelta::hours(14);
pub const MAX_BEHIND_UTC: TimeDelta = TimeDelta::hours(12);

pub type OpenSpan = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    Utc,
    Photo,
}

impl Clock {
    pub fn taken(self, asset: &AssetSummary) -> Option<DateTime<Utc>> {
        let raw = match self {
            Self::Utc => asset.file_created_at.as_deref(),
            Self::Photo => asset
                .local_date_time
                .as_deref()
                .or(asset.file_created_at.as_deref()),
        };
        parse_at(raw?)
    }

    pub fn past(self, filter: &BrowseFilter, at: DateTime<Utc>, later: bool) -> BrowseFilter {
        let (ahead, behind) = match self {
            Self::Utc => (TimeDelta::zero(), TimeDelta::zero()),
            Self::Photo => (MAX_AHEAD_OF_UTC, MAX_BEHIND_UTC),
        };
        let through = at + behind + TimeDelta::milliseconds(1);
        let span = if later {
            (Some(at - ahead), None)
        } else {
            (None, Some(through))
        };
        Self::Utc.bound(filter, span)
    }

    pub fn bound(self, filter: &BrowseFilter, (start, end): OpenSpan) -> BrowseFilter {
        match self {
            Self::Utc => BrowseFilter {
                taken_after: filter.taken_after.max(start),
                taken_until: earlier(filter.taken_until, end),
                ..filter.clone()
            },
            Self::Photo => {
                let picked = filter.taken_local.unwrap_or_default();
                let wall = |at: Option<DateTime<Utc>>| at.map(|at| at.naive_utc());
                let from = picked.from.max(wall(start));
                BrowseFilter {
                    taken_local: Some(LocalSpan {
                        from,
                        until: earlier(picked.until, wall(end)),
                    }),
                    ..filter.clone()
                }
            }
        }
    }
}

pub fn earlier<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

#[cfg(test)]
mod tests;
