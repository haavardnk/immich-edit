use super::*;
use crate::services::search::ids::IdSet;

fn ids(albums: IdSet, people: IdSet, tags: IdSet) -> Ids {
    Ids {
        albums,
        people,
        tags,
    }
}

fn all(id: Uuid) -> IdSet {
    IdSet {
        all: vec![id],
        ..IdSet::default()
    }
}

fn params(pairs: &[(&'static str, &str)]) -> Params {
    pairs
        .iter()
        .map(|(name, value)| (*name, (*value).to_owned()))
        .collect()
}

#[test]
fn the_timeline_is_narrowed_by_everything_it_can_filter() {
    let album = Uuid::new_v4();
    let person = Uuid::new_v4();
    let picked = Uuid::new_v4();
    let tag = Uuid::new_v4();
    let either = IdSet {
        any: vec![picked, tag],
        ..IdSet::default()
    };
    let cases = [
        (
            BrowseFilter {
                visibility: Some(Visibility::Timeline),
                ..BrowseFilter::default()
            },
            Ids::default(),
            Plan::Single(params(&[
                ("orderBy", "createdAt"),
                ("order", "desc"),
                ("visibility", "timeline"),
                ("withPartners", "true"),
            ])),
        ),
        (
            BrowseFilter {
                favorite: true,
                visibility: Some(Visibility::Archive),
                ..BrowseFilter::default()
            },
            Ids::default(),
            Plan::Single(params(&[
                ("orderBy", "createdAt"),
                ("order", "desc"),
                ("visibility", "archive"),
                ("isFavorite", "true"),
            ])),
        ),
        (
            BrowseFilter {
                favorite: true,
                visibility: Some(Visibility::Timeline),
                ..BrowseFilter::default()
            },
            ids(IdSet::default(), all(person), IdSet::default()),
            Plan::PerOwner {
                shared: params(&[
                    ("orderBy", "createdAt"),
                    ("order", "desc"),
                    ("personId", &person.to_string()),
                    ("visibility", "timeline"),
                    ("withPartners", "true"),
                ]),
                narrowed: params(&[
                    ("orderBy", "createdAt"),
                    ("order", "desc"),
                    ("personId", &person.to_string()),
                    ("visibility", "timeline"),
                    ("isFavorite", "true"),
                ]),
            },
        ),
        (
            BrowseFilter {
                favorite: true,
                visibility: Some(Visibility::Timeline),
                ..BrowseFilter::default()
            },
            ids(all(album), IdSet::default(), all(tag)),
            Plan::Single(params(&[
                ("orderBy", "createdAt"),
                ("order", "desc"),
                ("albumId", &album.to_string()),
                ("tagId", &tag.to_string()),
                ("visibility", "timeline"),
                ("isFavorite", "true"),
            ])),
        ),
        (
            BrowseFilter {
                scope: Some(Scope::Person(person)),
                visibility: Some(Visibility::Timeline),
                ..BrowseFilter::default()
            },
            ids(
                IdSet::default(),
                IdSet {
                    all: vec![picked, person],
                    ..IdSet::default()
                },
                either,
            ),
            Plan::Single(params(&[
                ("orderBy", "createdAt"),
                ("order", "desc"),
                ("personId", &person.to_string()),
                ("visibility", "timeline"),
                ("withPartners", "true"),
            ])),
        ),
        (
            BrowseFilter::default(),
            ids(
                IdSet {
                    any: vec![album],
                    none: vec![picked],
                    ..IdSet::default()
                },
                IdSet::default(),
                IdSet::default(),
            ),
            Plan::Single(params(&[
                ("orderBy", "createdAt"),
                ("order", "desc"),
                ("albumId", &album.to_string()),
            ])),
        ),
    ];
    for (filter, ids, expected) in cases {
        let planned = plan(&filter, &ids, Direction::Desc);
        if planned != expected {
            panic!("{filter:?} with {ids:?} planned {planned:?}, expected {expected:?}");
        }
    }
}

#[test]
fn the_order_follows_the_direction() {
    let planned = plan(&BrowseFilter::default(), &Ids::default(), Direction::Asc);
    let expected = Plan::Single(params(&[("orderBy", "createdAt"), ("order", "asc")]));
    if planned != expected {
        panic!("ascending planned {planned:?}");
    }
}
