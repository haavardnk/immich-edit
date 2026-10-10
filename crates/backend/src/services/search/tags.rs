use uuid::Uuid;

use crate::immich::dto::TagSummary;
use crate::services::search::filter::{BrowseFilter, Rejected, Scope};

pub const REJECT_TAG: &str = "immich-edit/reject";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TagSet {
    pub all: Vec<Uuid>,
    pub none: Vec<Uuid>,
}

pub fn needs_library(filter: &BrowseFilter) -> bool {
    filter.rejected.is_some()
}

pub fn tag_set(filter: &BrowseFilter, library: &[TagSummary]) -> Option<TagSet> {
    let mut set = TagSet::default();
    if let Some(Scope::Tag(id)) = filter.scope {
        set.all.push(id);
    }
    let reject = find(library, REJECT_TAG);
    match (filter.rejected, reject) {
        (Some(Rejected::Only), Some(id)) => set.all.push(id),
        (Some(Rejected::Only), None) => return None,
        (Some(Rejected::Hide), Some(id)) => set.none.push(id),
        (Some(Rejected::Hide), None) | (None, _) => {}
    }
    Some(set)
}

fn find(library: &[TagSummary], value: &str) -> Option<Uuid> {
    library
        .iter()
        .find(|tag| tag.value.eq_ignore_ascii_case(value))
        .map(|tag| tag.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(value: &str) -> TagSummary {
        TagSummary {
            id: Uuid::new_v4(),
            name: value.rsplit('/').next().unwrap_or(value).into(),
            value: value.into(),
            parent_id: None,
            color: None,
            created_at: None,
            updated_at: None,
            asset_count: None,
        }
    }

    #[test]
    fn rejected_filters_resolve_the_reject_tag() {
        let scope = Uuid::new_v4();
        let reject = tag("Immich-Edit/Reject");
        let library = vec![tag("immich-edit/label/red"), reject.clone()];
        let cases = [
            (None, vec![scope], vec![]),
            (Some(Rejected::Only), vec![scope, reject.id], vec![]),
            (Some(Rejected::Hide), vec![scope], vec![reject.id]),
        ];
        for (rejected, all, none) in cases {
            let filter_by = BrowseFilter {
                scope: Some(Scope::Tag(scope)),
                rejected,
                ..BrowseFilter::default()
            };
            if tag_set(&filter_by, &library) != Some(TagSet { all, none }) {
                panic!("wrong tag set for {rejected:?}");
            }
        }
    }

    #[test]
    fn a_missing_reject_tag_matches_nothing_or_hides_nothing() {
        let only = BrowseFilter {
            rejected: Some(Rejected::Only),
            ..BrowseFilter::default()
        };
        let hide = BrowseFilter {
            rejected: Some(Rejected::Hide),
            ..BrowseFilter::default()
        };
        if tag_set(&only, &[]).is_some() || tag_set(&hide, &[]) != Some(TagSet::default()) {
            panic!("a library without a reject tag resolved wrongly");
        }
    }
}
