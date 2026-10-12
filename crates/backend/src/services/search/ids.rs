use uuid::Uuid;

use crate::immich::dto::TagSummary;
use crate::services::search::filter::{BrowseFilter, Scope};
use crate::services::search::tags;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IdSet {
    pub all: Vec<Uuid>,
    pub any: Vec<Uuid>,
    pub none: Vec<Uuid>,
}

impl IdSet {
    fn required(&self) -> bool {
        !self.all.is_empty() || !self.any.is_empty()
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
    let mut ids = Ids::default();
    match filter.scope {
        Some(Scope::Album(id)) => ids.albums.all.push(id),
        Some(Scope::Person(id)) => ids.people.all.push(id),
        Some(Scope::Tag(id)) => ids.tags.all.push(id),
        None => {}
    }
    ids.tags.all.extend(managed.all);
    ids.tags.none.extend(managed.none);
    let contradicts = (filter.not_in_album && ids.albums.required())
        || (filter.untagged && ids.tags.required())
        || (filter.no_people && ids.people.required());
    (!contradicts).then_some(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::search::filter::Rejected;
    use crate::services::search::tags::REJECT_TAG;

    #[test]
    fn scope_and_managed_tags_resolve_together() {
        let scope = Uuid::new_v4();
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
            rejected: Some(Rejected::Hide),
            ..BrowseFilter::default()
        };
        let expected = Ids {
            tags: IdSet {
                all: vec![scope],
                none: vec![reject.id],
                ..IdSet::default()
            },
            ..Ids::default()
        };
        if resolve(&filter_by, &[reject]) != Some(expected) {
            panic!("scope and managed tags resolved wrongly");
        }
    }

    #[test]
    fn excluding_what_the_scope_requires_matches_nothing() {
        let id = Uuid::new_v4();
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
        ];
        for filter_by in cases {
            if resolve(&filter_by, &[]).is_some() {
                panic!("{filter_by:?} should match nothing");
            }
        }
    }
}
