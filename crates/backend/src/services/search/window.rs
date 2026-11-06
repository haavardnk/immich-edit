mod reach;
mod sides;

use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{Direction, PageQuery, Sort, SortField};
use crate::services::search::{Searcher, added, rating};
pub use reach::Reach;
pub(super) use sides::Side;
pub use sides::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Position {
    Taken(DateTime<Utc>),
    Size(u64),
}

type SortKey = (Position, Uuid);

fn sort_key(field: SortField, asset: &AssetSummary) -> Option<SortKey> {
    let position = if field == SortField::Size {
        Position::Size(asset.exif_info.as_ref()?.file_size_in_byte?)
    } else {
        let taken = DateTime::parse_from_rfc3339(asset.file_created_at.as_deref()?).ok()?;
        Position::Taken(taken.with_timezone(&Utc))
    };
    Some((position, asset.id.source()))
}

pub async fn around(
    searcher: &Searcher,
    filter: &BrowseFilter,
    sort: Sort,
    anchor: Uuid,
    reach: Reach,
) -> Result<Window, AppError> {
    let Some(anchor) = searcher.find(filter, anchor).await? else {
        return Ok(Window::default());
    };
    match sort.field {
        SortField::Added => {
            return added::around(searcher, filter, sort.direction, anchor, reach).await;
        }
        SortField::Rating => {
            return rating::around(searcher, filter, sort.direction, anchor, reach).await;
        }
        SortField::Taken | SortField::Size => {}
    }
    let Some(key) = sort_key(sort.field, &anchor) else {
        return Ok(Window::alone(anchor));
    };
    let (greater, lesser) = reach.greater_and_lesser(sort.direction);
    let greater = side(searcher, filter, sort, key, Ordering::Greater, greater).await?;
    let lesser = side(searcher, filter, sort, key, Ordering::Less, lesser).await?;
    Ok(Window::by_key(sort.direction, greater, anchor, lesser))
}

async fn side(
    searcher: &Searcher,
    filter: &BrowseFilter,
    sort: Sort,
    anchor: SortKey,
    toward: Ordering,
    wanted: usize,
) -> Result<Side, AppError> {
    let mut bounded = filter.clone();
    match (anchor.0, toward) {
        (Position::Taken(at), Ordering::Greater) => {
            bounded.taken_after = Some(filter.taken_after.map_or(at, |after| after.max(at)));
        }
        (Position::Taken(at), _) => {
            bounded.taken_before = Some(filter.taken_before.map_or(at, |before| before.min(at)));
        }
        (Position::Size(bytes), Ordering::Greater) => {
            bounded.min_size = Some(filter.min_size.map_or(bytes, |min| min.max(bytes)));
        }
        (Position::Size(bytes), _) => {
            bounded.max_size = Some(filter.max_size.map_or(bytes, |max| max.min(bytes)));
        }
    }
    let direction = match toward {
        Ordering::Greater => Direction::Asc,
        _ => Direction::Desc,
    };
    let mut query = PageQuery {
        filter: bounded,
        sort: Sort { direction, ..sort },
        size: wanted as u32 + 2,
        cursor: None,
        with_exif: true,
    };
    let mut out: Vec<AssetSummary> = Vec::new();
    loop {
        let page = searcher.page(&query).await?;
        out.extend(page.items.into_iter().filter(|asset| {
            sort_key(sort.field, asset).is_some_and(|key| key.cmp(&anchor) == toward)
        }));
        if out.len() > wanted {
            break;
        }
        let Some(cursor) = page.next_cursor else {
            break;
        };
        query.cursor = Some(cursor);
    }
    Ok(Side::cut(out, wanted))
}

#[cfg(test)]
pub(super) mod tests;
