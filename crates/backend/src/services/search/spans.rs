use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::search::Searcher;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::months::{Span, counts};

pub const MAX_EDGES: usize = 64;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpanQuery {
    #[serde(default)]
    pub filter: BrowseFilter,
    pub edges: Vec<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SpanCounts {
    pub counts: Vec<u64>,
}

pub async fn count(searcher: &Searcher, query: &SpanQuery) -> Result<SpanCounts, AppError> {
    let spans = spans(&query.edges)?;
    Ok(SpanCounts {
        counts: counts(searcher, &query.filter, spans).await?,
    })
}

fn spans(edges: &[DateTime<Utc>]) -> Result<Vec<Span>, AppError> {
    let increasing = edges.windows(2).all(|pair| pair[0] < pair[1]);
    if !(2..=MAX_EDGES).contains(&edges.len()) || !increasing {
        return Err(AppError::BadRequest(format!(
            "edges must be 2 to {MAX_EDGES} increasing instants"
        )));
    }
    Ok(edges.windows(2).map(|pair| (pair[0], pair[1])).collect())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone};

    use super::*;

    #[test]
    fn consecutive_edges_become_spans_and_bad_edges_are_rejected() {
        let start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
        let day = TimeDelta::days(1);
        let found = spans(&[start, start + day, start + day * 2]).ok();
        if found != Some(vec![(start, start + day), (start + day, start + day * 2)]) {
            panic!("spans were {found:?}");
        }
        let too_many: Vec<DateTime<Utc>> =
            Vec::from_fn(MAX_EDGES + 1, |at| start + TimeDelta::hours(at as i64));
        for edges in [
            vec![start],
            vec![start, start],
            vec![start + day, start],
            too_many,
        ] {
            if spans(&edges).is_ok() {
                panic!("{} edges were accepted", edges.len());
            }
        }
    }
}
