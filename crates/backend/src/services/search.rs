pub mod cursor;
pub mod filter;
mod flat;
pub mod query;
mod structured;
pub mod window;

use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::{AssetSummary, SearchAssets};
use crate::services::immich_capabilities::CapabilityCache;
use cursor::Cursor;
use filter::BrowseFilter;
use query::{PageQuery, SmartQuery, check_size};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub items: Vec<AssetSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchShape {
    Flat,
    Structured,
}

#[derive(Clone)]
pub struct Searcher {
    immich: ImmichClient,
    shape: SearchShape,
}

impl Searcher {
    pub fn new(immich: ImmichClient, shape: SearchShape) -> Self {
        Self { immich, shape }
    }

    pub async fn connect(
        capabilities: &CapabilityCache,
        server_epoch: i64,
        immich: &ImmichClient,
    ) -> Self {
        let shape = if capabilities
            .get(server_epoch, immich)
            .await
            .structured_search
        {
            SearchShape::Structured
        } else {
            SearchShape::Flat
        };
        Self::new(immich.clone(), shape)
    }

    pub async fn page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
        if self.shape == SearchShape::Flat {
            let page = flat_page(cursor)?;
            let assets = self
                .immich
                .search_metadata(&flat::page(query, page))
                .await?;
            return Ok(flat_result(assets));
        }
        let token = match cursor {
            None => None,
            Some(Cursor::Immich { token }) => Some(token),
            Some(_) => return Err(Cursor::mismatch()),
        };
        let assets = self
            .immich
            .search_metadata(&structured::page(query, token.as_deref()))
            .await?;
        Ok(SearchPage {
            items: assets.items,
            next_cursor: assets
                .next_cursor
                .map(|token| Cursor::Immich { token }.encode()),
        })
    }

    pub async fn smart(&self, query: &SmartQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
        let page = flat_page(cursor)?;
        let assets = self.immich.search_smart(&flat::smart(query, page)).await?;
        Ok(flat_result(assets))
    }

    pub async fn count(&self, filter: &BrowseFilter) -> Result<u64, AppError> {
        let body = match self.shape {
            SearchShape::Flat => flat::count(filter),
            SearchShape::Structured => structured::count(filter),
        };
        Ok(self.immich.search_statistics(&body).await?.total)
    }

    pub async fn find(
        &self,
        filter: &BrowseFilter,
        id: Uuid,
    ) -> Result<Option<AssetSummary>, AppError> {
        let body = match self.shape {
            SearchShape::Flat => flat::find(filter, id),
            SearchShape::Structured => structured::find(filter, id),
        };
        let found = self.immich.search_metadata(&body).await?;
        Ok(found.items.into_iter().next())
    }
}

fn flat_page(cursor: Option<Cursor>) -> Result<u32, AppError> {
    match cursor {
        None => Ok(1),
        Some(Cursor::Page { page }) => Ok(page),
        Some(_) => Err(Cursor::mismatch()),
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
