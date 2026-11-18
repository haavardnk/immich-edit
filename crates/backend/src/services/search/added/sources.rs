use uuid::Uuid;

use crate::services::search::filter::{BrowseFilter, Scope, Visibility};
use crate::services::search::ids::{IdSet, Ids};
use crate::services::search::query::Direction;

pub(super) type Params = Vec<(&'static str, String)>;

pub(super) fn params(filter: &BrowseFilter, ids: &Ids, direction: Direction) -> Params {
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
    if filter.favorite {
        params.push(("isFavorite", "true".into()));
    } else if filter.visibility == Some(Visibility::Timeline) && !in_album {
        params.push(("withPartners", "true".into()));
    }
    params
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
