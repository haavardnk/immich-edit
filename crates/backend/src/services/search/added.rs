mod buckets;
mod hydrate;
mod sources;
mod timeline;

use chrono::{DateTime, Datelike, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::cursor::Cursor;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{Direction, PageQuery};
use crate::services::search::window::{Reach, Side, Window};
use crate::services::search::{SearchPage, Searcher};
pub use buckets::TimelineCache;
pub(super) use buckets::parse_at;
use timeline::{Position, Timeline};

const SIDE_CHUNK: usize = 64;

pub async fn page(searcher: &Searcher, query: &PageQuery) -> Result<SearchPage, AppError> {
    let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
    let Some(ids) = searcher.ids(&query.filter).await? else {
        return Ok(SearchPage::empty());
    };
    let timeline = Timeline::load(searcher, &query.filter, &ids, query.sort.direction).await?;
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
    reach: Reach,
) -> Result<Window, AppError> {
    let Some(at) = anchor.created_at.as_deref().and_then(parse_at) else {
        return Ok(Window::alone(anchor));
    };
    let Some(ids) = searcher.ids(filter).await? else {
        return Ok(Window::alone(anchor));
    };
    let bucket = format!("{:04}-{:02}-01", at.year(), at.month());
    let id = anchor.id.source();
    let after = Timeline::load(searcher, filter, &ids, direction).await?;
    let before = after.reversed();
    let before = side(&before, filter, (&bucket, at, id), reach.before as usize).await?;
    let after = side(&after, filter, (&bucket, at, id), reach.after as usize).await?;
    Ok(Window::around(before, anchor, after))
}

async fn side(
    timeline: &Timeline<'_>,
    filter: &BrowseFilter,
    (bucket, at, id): (&str, DateTime<Utc>, Uuid),
    wanted: usize,
) -> Result<Side, AppError> {
    let mut start = timeline.resume(bucket, at, id).await?;
    let mut items = Vec::new();
    loop {
        let size = (wanted + 1 - items.len()).max(SIDE_CHUNK);
        let scan = timeline.scan(start, size, filter, true).await?;
        items.extend(scan.items);
        if items.len() > wanted {
            break;
        }
        let Some(Cursor::Added { bucket, at, id }) = scan.resume else {
            break;
        };
        start = timeline.resume(&bucket, at, id).await?;
    }
    Ok(Side::cut(items, wanted))
}

#[cfg(test)]
mod tests;
