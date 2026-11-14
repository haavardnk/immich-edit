use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::services::search::Searcher;
use crate::services::search::filter::{BrowseFilter, Scope, Visibility};
use crate::services::search::ids::{IdSet, Ids};
use crate::services::search::query::Direction;

pub(super) type Params = Vec<(&'static str, String)>;

#[derive(Debug, PartialEq, Eq)]
enum Plan {
    Single(Params),
    PerOwner { shared: Params, narrowed: Params },
}

pub(super) async fn sources(
    searcher: &Searcher,
    filter: &BrowseFilter,
    ids: &Ids,
    direction: Direction,
) -> Result<Vec<Params>, AppError> {
    let (shared, narrowed) = match plan(filter, ids, direction) {
        Plan::Single(params) => return Ok(vec![params]),
        Plan::PerOwner { shared, narrowed } => (shared, narrowed),
    };
    let partners = match searcher.immich.timeline_partners().await {
        Ok(partners) => partners,
        Err(ImmichError::Rejected { status: 403, .. }) => return Ok(vec![shared]),
        Err(err) => return Err(err.into()),
    };
    let owned = partners.iter().map(|partner| {
        let mut params = narrowed.clone();
        params.push(("userId", partner.id.to_string()));
        params
    });
    Ok(std::iter::once(narrowed.clone()).chain(owned).collect())
}

fn plan(filter: &BrowseFilter, ids: &Ids, direction: Direction) -> Plan {
    let order = match direction {
        Direction::Asc => "asc",
        Direction::Desc => "desc",
    };
    let mut params: Params = vec![("orderBy", "createdAt".into()), ("order", order.into())];
    let (album, person, tag) = match filter.scope {
        Some(Scope::Album(id)) => (Some(id), None, None),
        Some(Scope::Person(id)) => (None, Some(id), None),
        Some(Scope::Tag(id)) => (None, None, Some(id)),
        None => (None, None, None),
    };
    let slots = [
        ("albumId", required(&ids.albums, album)),
        ("personId", required(&ids.people, person)),
        ("tagId", required(&ids.tags, tag)),
    ];
    let in_album = slots[0].1.is_some();
    for (name, id) in slots {
        if let Some(id) = id {
            params.push((name, id.to_string()));
        }
    }
    let visibility = filter.visibility.map(|visibility| match visibility {
        Visibility::Timeline => "timeline",
        Visibility::Archive => "archive",
        Visibility::Hidden => "hidden",
    });
    if let Some(visibility) = visibility {
        params.push(("visibility", visibility.into()));
    }
    let partners = filter.visibility == Some(Visibility::Timeline) && !in_album;
    let mut shared = params.clone();
    if partners {
        shared.push(("withPartners", "true".into()));
    }
    if !filter.favorite {
        return Plan::Single(shared);
    }
    params.push(("isFavorite", "true".into()));
    if partners {
        Plan::PerOwner {
            shared,
            narrowed: params,
        }
    } else {
        Plan::Single(params)
    }
}

fn required(set: &IdSet, scoped: Option<Uuid>) -> Option<Uuid> {
    scoped
        .or_else(|| set.all.first().copied())
        .or(match set.any.as_slice() {
            [only] => Some(*only),
            _ => None,
        })
}

#[cfg(test)]
mod tests;
