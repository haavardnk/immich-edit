use serde::Deserialize;
use uuid::Uuid;

use crate::immich::dto::TagSummary;
use crate::services::search::filter::{BrowseFilter, Scope};
use crate::services::search::tags;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IdSet {
    pub all: Vec<Uuid>,
    pub any: Vec<Uuid>,
    pub none: Vec<Uuid>,
}

impl IdSet {
    fn required(&self) -> bool {
        !self.all.is_empty() || !self.any.is_empty()
    }

    fn require(&mut self, ids: impl IntoIterator<Item = Uuid>) {
        push_new(&mut self.all, ids);
    }

    fn exclude(&mut self, ids: impl IntoIterator<Item = Uuid>) {
        push_new(&mut self.none, ids);
    }

    fn contradicts(&self) -> bool {
        self.all.iter().any(|id| self.none.contains(id))
            || (!self.any.is_empty() && self.any.iter().all(|id| self.none.contains(id)))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Ids {
    pub albums: IdSet,
    pub people: IdSet,
    pub tags: IdSet,
}

impl Ids {
    pub fn fields(&self) -> [(&'static str, &IdSet); 3] {
        [
            ("albumIds", &self.albums),
            ("personIds", &self.people),
            ("tagIds", &self.tags),
        ]
    }
}

pub fn resolve(filter: &BrowseFilter, library: &[TagSummary]) -> Option<Ids> {
    let managed = tags::managed(filter, library)?;
    let mut ids = Ids {
        albums: filter.albums.clone(),
        people: filter.people.clone(),
        tags: filter.tags.clone(),
    };
    match filter.scope {
        Some(Scope::Album(id)) => ids.albums.require([id]),
        Some(Scope::Person(id)) => ids.people.require([id]),
        Some(Scope::Tag(id)) => ids.tags.require([id]),
        None => {}
    }
    ids.tags.require(managed.all);
    ids.tags.exclude(managed.none);
    let contradicts = (filter.not_in_album && ids.albums.required())
        || (filter.untagged && ids.tags.required())
        || (filter.no_people && ids.people.required())
        || ids.fields().into_iter().any(|(_, set)| set.contradicts());
    (!contradicts).then_some(ids)
}

fn push_new(list: &mut Vec<Uuid>, ids: impl IntoIterator<Item = Uuid>) {
    for id in ids {
        if !list.contains(&id) {
            list.push(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::search::filter::Rejected;
    use crate::services::search::tags::REJECT_TAG;

    #[test]
    fn scope_picked_and_managed_tags_resolve_together() {
        let scope = Uuid::new_v4();
        let picked = Uuid::new_v4();
        let reject = TagSummary {
            id: Uuid::new_v4(),
            name: "reject".into(),
            value: REJECT_TAG.into(),
            parent_id: None,
            color: None,
            created_at: None,
            updated_at: None,
            asset_count: None,
        };
        let filter_by = BrowseFilter {
            scope: Some(Scope::Tag(scope)),
            tags: IdSet {
                any: vec![picked],
                none: vec![reject.id],
                ..IdSet::default()
            },
            rejected: Some(Rejected::Hide),
            ..BrowseFilter::default()
        };
        let expected = Ids {
            tags: IdSet {
                all: vec![scope],
                any: vec![picked],
                none: vec![reject.id],
            },
            ..Ids::default()
        };
        if resolve(&filter_by, &[reject]) != Some(expected) {
            panic!("scope and managed tags resolved wrongly");
        }
    }

    #[test]
    fn picked_people_join_the_scope_person_once() {
        let scope = Uuid::new_v4();
        let picked = Uuid::new_v4();
        let filter_by = BrowseFilter {
            scope: Some(Scope::Person(scope)),
            people: IdSet {
                all: vec![scope],
                any: vec![picked],
                ..IdSet::default()
            },
            ..BrowseFilter::default()
        };
        let people = resolve(&filter_by, &[]).map(|ids| ids.people);
        let expected = IdSet {
            all: vec![scope],
            any: vec![picked],
            ..IdSet::default()
        };
        if people != Some(expected) {
            panic!("people resolved as {people:?}");
        }
    }

    #[test]
    fn excluding_what_the_scope_requires_matches_nothing() {
        let id = Uuid::new_v4();
        let other = Uuid::new_v4();
        let cases = [
            BrowseFilter {
                scope: Some(Scope::Album(id)),
                not_in_album: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                scope: Some(Scope::Tag(id)),
                untagged: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                scope: Some(Scope::Person(id)),
                no_people: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                albums: IdSet {
                    all: vec![id],
                    ..IdSet::default()
                },
                not_in_album: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                people: IdSet {
                    any: vec![id],
                    ..IdSet::default()
                },
                no_people: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                scope: Some(Scope::Person(id)),
                people: IdSet {
                    none: vec![id],
                    ..IdSet::default()
                },
                ..BrowseFilter::default()
            },
            BrowseFilter {
                people: IdSet {
                    any: vec![id, other],
                    none: vec![other, id],
                    ..IdSet::default()
                },
                ..BrowseFilter::default()
            },
        ];
        for filter_by in cases {
            if resolve(&filter_by, &[]).is_some() {
                panic!("{filter_by:?} should match nothing");
            }
        }
    }
}
