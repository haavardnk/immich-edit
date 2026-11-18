use std::collections::{HashMap, HashSet};

use chrono::TimeDelta;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::AssetSummary;
use crate::services::search::Searcher;
use crate::services::search::added::buckets::Entry;
use crate::services::search::filter::BrowseFilter;
use crate::services::search::query::{MAX_SIZE, PageQuery, Sort};

pub(super) async fn hydrate(
    searcher: &Searcher,
    filter: &BrowseFilter,
    with_exif: bool,
    chunk: &[(usize, Entry)],
) -> Result<HashMap<Uuid, AssetSummary>, AppError> {
    let times = || chunk.iter().map(|(_, entry)| entry.at);
    let (Some(low), Some(high)) = (times().min(), times().max()) else {
        return Ok(HashMap::new());
    };
    let bounded = BrowseFilter {
        added_between: Some((
            low - TimeDelta::milliseconds(1),
            high + TimeDelta::milliseconds(1),
        )),
        ..filter.clone()
    };
    if let (Some(after), Some(before)) = bounded.added_bounds()
        && after > before
    {
        return Ok(HashMap::new());
    }
    let wanted: HashSet<Uuid> = chunk.iter().map(|(_, entry)| entry.id).collect();
    let mut query = PageQuery {
        filter: bounded,
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
