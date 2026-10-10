mod added;
pub mod cursor;
pub mod filter;
mod flat;
pub mod query;
mod structured;
mod tags;
pub mod window;

use std::sync::Arc;

use serde::Serialize;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::{AssetSummary, SearchAssets, TagSummary};
use crate::services::immich_capabilities::CapabilityCache;
pub use added::TimelineCache;
use cursor::Cursor;
use filter::BrowseFilter;
use query::{PageQuery, SmartQuery, SortField, check_size};
use tags::TagSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub items: Vec<AssetSummary>,
    pub next_cursor: Option<String>,
}

impl SearchPage {
    fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
        }
    }
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
    library: Arc<OnceCell<Vec<TagSummary>>>,
    timelines: TimelineCache,
    owner: Uuid,
    server_epoch: i64,
}

impl Searcher {
    pub fn new(immich: ImmichClient, shape: SearchShape) -> Self {
        Self {
            immich,
            shape,
            library: Arc::new(OnceCell::new()),
            timelines: TimelineCache::default(),
            owner: Uuid::nil(),
            server_epoch: 0,
        }
    }

    pub async fn connect(
        capabilities: &CapabilityCache,
        timelines: &TimelineCache,
        owner: Uuid,
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
        Self {
            timelines: timelines.clone(),
            owner,
            server_epoch,
            ..Self::new(immich.clone(), shape)
        }
    }

    pub async fn page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        if query.sort.field == SortField::Added {
            return added::page(self, query).await;
        }
        self.metadata_page(query).await
    }

    async fn metadata_page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
        let Some(tags) = self.tags(&query.filter).await? else {
            return Ok(SearchPage::empty());
        };
        if self.shape == SearchShape::Flat {
            let page = flat_page(cursor)?;
            let assets = self
                .immich
                .search_metadata(&flat::page(query, &tags, page)?)
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
            .search_metadata(&structured::page(query, &tags, token.as_deref()))
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
        let Some(tags) = self.tags(&query.filter).await? else {
            return Ok(SearchPage::empty());
        };
        let page = flat_page(cursor)?;
        let assets = self
            .immich
            .search_smart(&flat::smart(query, &tags, page)?)
            .await?;
        Ok(flat_result(assets))
    }

    pub async fn count(&self, filter: &BrowseFilter) -> Result<u64, AppError> {
        let Some(tags) = self.tags(filter).await? else {
            return Ok(0);
        };
        let body = match self.shape {
            SearchShape::Flat => flat::count(filter, &tags)?,
            SearchShape::Structured => structured::count(filter, &tags),
        };
        Ok(self.immich.search_statistics(&body).await?.total)
    }

    pub async fn find(
        &self,
        filter: &BrowseFilter,
        id: Uuid,
    ) -> Result<Option<AssetSummary>, AppError> {
        let Some(tags) = self.tags(filter).await? else {
            return Ok(None);
        };
        let body = match self.shape {
            SearchShape::Flat => flat::find(filter, &tags, id)?,
            SearchShape::Structured => structured::find(filter, &tags, id),
        };
        let found = self.immich.search_metadata(&body).await?;
        Ok(found.items.into_iter().next())
    }

    async fn tags(&self, filter: &BrowseFilter) -> Result<Option<TagSet>, AppError> {
        if !tags::needs_library(filter) {
            return Ok(tags::tag_set(filter, &[]));
        }
        let library = self
            .library
            .get_or_try_init(|| self.immich.list_tags())
            .await?;
        Ok(tags::tag_set(filter, library))
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
