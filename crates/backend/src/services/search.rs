pub mod cursor;
pub mod filter;
mod flat;
pub mod query;
pub mod window;

use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::{AssetSummary, SearchAssets};
use cursor::Cursor;
use filter::BrowseFilter;
use query::{PageQuery, SmartQuery, check_size};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub items: Vec<AssetSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Clone)]
pub struct Searcher {
    immich: ImmichClient,
}

impl Searcher {
    pub fn new(immich: ImmichClient) -> Self {
        Self { immich }
    }

    pub async fn page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        let page = flat_page(query.cursor.as_deref())?;
        let assets = self
            .immich
            .search_metadata(&flat::page(query, page))
            .await?;
        Ok(flat_result(assets))
    }

    pub async fn smart(&self, query: &SmartQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        let page = flat_page(query.cursor.as_deref())?;
        let assets = self.immich.search_smart(&flat::smart(query, page)).await?;
        Ok(flat_result(assets))
    }

    pub async fn count(&self, filter: &BrowseFilter) -> Result<u64, AppError> {
        let stats = self.immich.search_statistics(&flat::count(filter)).await?;
        Ok(stats.total)
    }

    pub async fn find(
        &self,
        filter: &BrowseFilter,
        id: Uuid,
    ) -> Result<Option<AssetSummary>, AppError> {
        let found = self.immich.search_metadata(&flat::find(filter, id)).await?;
        Ok(found.items.into_iter().next())
    }
}

fn flat_page(cursor: Option<&str>) -> Result<u32, AppError> {
    match cursor.map(Cursor::decode).transpose()? {
        None => Ok(1),
        Some(Cursor::Page { page }) => Ok(page),
    }
}

fn flat_result(assets: SearchAssets) -> SearchPage {
    let next_cursor = assets
        .next_page
        .and_then(|page| page.parse().ok())
        .map(|page| Cursor::Page { page }.encode());
    SearchPage {
        items: assets.items,
        next_cursor,
    }
}
