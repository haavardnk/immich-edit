mod added;
mod caches;
pub mod clock;
pub mod cursor;
pub mod filter;
mod flat;
mod ids;
pub mod local_taken;
pub mod locate;
pub mod months;
pub mod query;
mod rating;
pub mod spans;
mod structured;
pub mod suggestions;
mod tags;
pub mod window;

use std::collections::HashSet;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::{AssetSummary, SearchAssets, TagSummary};
use crate::services::immich_capabilities::CapabilityCache;
pub use caches::SearchCaches;
use clock::Clock;
use cursor::Cursor;
use filter::BrowseFilter;
use ids::Ids;
use query::{PageQuery, SmartQuery, SortField, check_size};

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

    fn emptied_by(&mut self, excluded: &HashSet<Uuid>) -> bool {
        if excluded.is_empty() {
            return false;
        }
        self.items
            .retain(|asset| !excluded.contains(&asset.id.source()));
        self.items.is_empty() && self.next_cursor.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchShape {
    Flat,
    Structured,
}

struct Prepared {
    filter: BrowseFilter,
    ids: Ids,
    excluded: HashSet<Uuid>,
}

#[derive(Clone)]
pub struct Searcher {
    immich: ImmichClient,
    shape: SearchShape,
    library: Arc<OnceCell<Vec<TagSummary>>>,
    caches: SearchCaches,
    owner: Uuid,
    server_epoch: i64,
}

impl Searcher {
    pub fn new(immich: ImmichClient, shape: SearchShape) -> Self {
        Self {
            immich,
            shape,
            library: Arc::new(OnceCell::new()),
            caches: SearchCaches::default(),
            owner: Uuid::nil(),
            server_epoch: 0,
        }
    }

    pub async fn connect(
        capabilities: &CapabilityCache,
        caches: &SearchCaches,
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
            caches: caches.clone(),
            owner,
            server_epoch,
            ..Self::new(immich.clone(), shape)
        }
    }

    pub fn clock(&self) -> Clock {
        match self.shape {
            SearchShape::Flat => Clock::Utc,
            SearchShape::Structured => Clock::Photo,
        }
    }

    pub async fn page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        check_size(query.size)?;
        match query.sort.field {
            SortField::Added => added::page(self, query).await,
            SortField::Rating | SortField::Size if self.shape == SearchShape::Flat => Err(
                AppError::BadRequest("this server cannot sort by rating or size".into()),
            ),
            SortField::Taken | SortField::Rating | SortField::Size => {
                self.metadata_page(query).await
            }
        }
    }

    async fn metadata_page(&self, query: &PageQuery) -> Result<SearchPage, AppError> {
        let Some(prepared) = self.prepare(&query.filter).await? else {
            return Ok(SearchPage::empty());
        };
        let mut query = PageQuery {
            filter: prepared.filter,
            ..query.clone()
        };
        loop {
            let mut page = self.immich_page(&query, &prepared.ids).await?;
            if !page.emptied_by(&prepared.excluded) {
                return Ok(page);
            }
            query.cursor = page.next_cursor;
        }
    }

    async fn immich_page(&self, query: &PageQuery, ids: &Ids) -> Result<SearchPage, AppError> {
        let cursor = query.cursor.as_deref().map(Cursor::decode).transpose()?;
        if self.shape == SearchShape::Flat {
            let page = flat_page(cursor)?;
            let assets = self
                .immich
                .search_metadata(&flat::page(query, ids, page)?)
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
            .search_metadata(&structured::page(query, ids, token.as_deref()))
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
        let Some(prepared) = self.prepare(&query.filter).await? else {
            return Ok(SearchPage::empty());
        };
        let mut query = SmartQuery {
            filter: prepared.filter,
            ..query.clone()
        };
        loop {
            let page = flat_page(query.cursor.as_deref().map(Cursor::decode).transpose()?)?;
            let assets = self
                .immich
                .search_smart(&flat::smart(&query, &prepared.ids, page)?)
                .await?;
            let mut found = flat_result(assets);
            if !found.emptied_by(&prepared.excluded) {
                return Ok(found);
            }
            query.cursor = found.next_cursor;
        }
    }

    pub async fn count(&self, filter: &BrowseFilter) -> Result<u64, AppError> {
        let Some(prepared) = self.prepare(filter).await? else {
            return Ok(0);
        };
        let body = self.count_body(&prepared)?;
        let total = self.immich.search_statistics(&body).await?.total;
        Ok(total.saturating_sub(prepared.excluded.len() as u64))
    }

    async fn count_key(&self, filter: &BrowseFilter) -> Result<Option<String>, AppError> {
        let Some(prepared) = self.prepare(filter).await? else {
            return Ok(None);
        };
        let mut excluded: Vec<Uuid> = prepared.excluded.iter().copied().collect();
        excluded.sort_unstable();
        Ok(Some(format!("{}{excluded:?}", self.count_body(&prepared)?)))
    }

    fn count_body(&self, prepared: &Prepared) -> Result<Value, AppError> {
        match self.shape {
            SearchShape::Flat => flat::count(&prepared.filter, &prepared.ids),
            SearchShape::Structured => Ok(structured::count(&prepared.filter, &prepared.ids)),
        }
    }

    pub async fn find(
        &self,
        filter: &BrowseFilter,
        id: Uuid,
    ) -> Result<Option<AssetSummary>, AppError> {
        let Some(prepared) = self.prepare(filter).await? else {
            return Ok(None);
        };
        if prepared.excluded.contains(&id) {
            return Ok(None);
        }
        let body = match self.shape {
            SearchShape::Flat => flat::find(&prepared.filter, &prepared.ids, id)?,
            SearchShape::Structured => structured::find(&prepared.filter, &prepared.ids, id),
        };
        let found = self.immich.search_metadata(&body).await?;
        Ok(found.items.into_iter().next())
    }

    async fn prepare(&self, filter: &BrowseFilter) -> Result<Option<Prepared>, AppError> {
        let Some(ids) = self.ids(filter).await? else {
            return Ok(None);
        };
        let Some(span) = filter.taken_local else {
            return Ok(Some(Prepared {
                filter: filter.clone(),
                ids,
                excluded: HashSet::new(),
            }));
        };
        let Some(bounds) = local_taken::bounds(self, filter, &ids, span).await? else {
            return Ok(None);
        };
        let narrowed = bounds.narrow(filter);
        let excluded = bounds.excluded_by(&narrowed);
        Ok(Some(Prepared {
            filter: narrowed,
            ids,
            excluded,
        }))
    }

    async fn ids(&self, filter: &BrowseFilter) -> Result<Option<Ids>, AppError> {
        if !tags::needs_library(filter) {
            return Ok(ids::resolve(filter, &[]));
        }
        let library = self
            .library
            .get_or_try_init(|| self.immich.list_tags())
            .await?;
        Ok(ids::resolve(filter, library))
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
