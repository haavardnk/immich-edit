use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::services::search::filter::{BrowseFilter, RatingMatch, Scope};
use crate::services::search::flat::timestamp;
use crate::services::search::query::PageQuery;
use crate::services::search::tags::TagSet;

pub fn filter(filter: &BrowseFilter, tags: &TagSet) -> Map<String, Value> {
    let mut body = Map::new();
    body.insert("trashedAt".into(), json!({ "eq": null }));
    match filter.scope {
        Some(Scope::Album(id)) => body.insert("albumIds".into(), json!({ "all": [id] })),
        Some(Scope::Person(id)) => body.insert("personIds".into(), json!({ "all": [id] })),
        Some(Scope::Tag(_)) | None => None,
    };
    let mut tag_ids = Map::new();
    if !tags.all.is_empty() {
        tag_ids.insert("all".into(), json!(tags.all));
    }
    if !tags.none.is_empty() {
        tag_ids.insert("none".into(), json!(tags.none));
    }
    if !tag_ids.is_empty() {
        body.insert("tagIds".into(), Value::Object(tag_ids));
    }
    if filter.favorite {
        body.insert("isFavorite".into(), json!({ "eq": true }));
    }
    if let Some(visibility) = filter.visibility {
        body.insert("visibility".into(), json!({ "eq": visibility }));
    }
    if filter.images_only {
        body.insert("type".into(), json!({ "eq": "IMAGE" }));
    }
    match filter.rating {
        Some(RatingMatch::Eq(stars)) => body.insert("rating".into(), json!({ "eq": stars })),
        Some(RatingMatch::Gte(stars)) => body.insert("rating".into(), json!({ "gte": stars })),
        Some(RatingMatch::Lte(stars)) => body.insert(
            "or".into(),
            json!([{ "rating": { "lte": stars } }, { "rating": { "eq": null } }]),
        ),
        Some(RatingMatch::Unrated) => body.insert("rating".into(), json!({ "eq": null })),
        None => None,
    };
    if let Some(name) = filter.filename.as_deref().map(str::trim)
        && !name.is_empty()
    {
        body.insert("originalFileName".into(), json!({ "like": name }));
    }
    let mut taken = Map::new();
    if let Some(after) = filter.taken_after {
        taken.insert("gte".into(), timestamp(after));
    }
    if let Some(before) = filter.taken_before {
        taken.insert("lte".into(), timestamp(before));
    }
    if !taken.is_empty() {
        body.insert("takenAt".into(), Value::Object(taken));
    }
    if let Some((after, before)) = filter.added_between {
        body.insert(
            "createdAt".into(),
            json!({ "gte": timestamp(after), "lte": timestamp(before) }),
        );
    }
    body
}

pub fn page(query: &PageQuery, tags: &TagSet, cursor: Option<&str>) -> Value {
    let mut body = json!({
        "filter": filter(&query.filter, tags),
        "orderBy": { "field": "fileCreatedAt", "direction": query.sort.direction },
        "size": query.size,
    });
    if let Some(token) = cursor {
        body["cursor"] = token.into();
    }
    if query.with_exif {
        body["withExif"] = true.into();
    }
    body
}

pub fn find(filter_by: &BrowseFilter, tags: &TagSet, id: Uuid) -> Value {
    let mut conditions = filter(filter_by, tags);
    conditions.insert("id".into(), json!({ "eq": id }));
    json!({ "filter": conditions, "size": 1, "withExif": true })
}

pub fn count(filter_by: &BrowseFilter, tags: &TagSet) -> Value {
    json!({ "filter": filter(filter_by, tags) })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::services::search::filter::Visibility;
    use crate::services::search::query::{Direction, Sort, SortField};

    #[test]
    fn a_browse_page_maps_to_a_structured_immich_filter() {
        let album = Uuid::new_v4();
        let tags = TagSet {
            all: vec![Uuid::new_v4()],
            none: vec![Uuid::new_v4()],
        };
        let query = PageQuery {
            filter: BrowseFilter {
                scope: Some(Scope::Album(album)),
                favorite: true,
                visibility: Some(Visibility::Timeline),
                images_only: true,
                rating: Some(RatingMatch::Eq(3)),
                filename: Some(" DSC ".into()),
                taken_after: None,
                taken_before: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
                rejected: None,
                label: None,
                added_between: None,
            },
            sort: Sort {
                field: SortField::Taken,
                direction: Direction::Asc,
            },
            size: 500,
            cursor: None,
            with_exif: true,
        };
        let expected = json!({
            "filter": {
                "trashedAt": { "eq": null },
                "albumIds": { "all": [album] },
                "tagIds": { "all": tags.all, "none": tags.none },
                "isFavorite": { "eq": true },
                "visibility": { "eq": "timeline" },
                "type": { "eq": "IMAGE" },
                "rating": { "eq": 3 },
                "originalFileName": { "like": "DSC" },
                "takenAt": { "lte": "2024-01-01T00:00:00.000Z" },
            },
            "orderBy": { "field": "fileCreatedAt", "direction": "asc" },
            "size": 500,
            "cursor": "next",
            "withExif": true,
        });
        let body = page(&query, &tags, Some("next"));
        if body != expected {
            panic!("structured body was {body}");
        }
    }

    #[test]
    fn a_window_probe_matches_one_asset_by_id() {
        let id = Uuid::new_v4();
        let filter_by = BrowseFilter {
            rating: Some(RatingMatch::Unrated),
            ..BrowseFilter::default()
        };
        let expected = json!({
            "filter": {
                "trashedAt": { "eq": null },
                "rating": { "eq": null },
                "id": { "eq": id },
            },
            "size": 1,
            "withExif": true,
        });
        let body = find(&filter_by, &TagSet::default(), id);
        if body != expected {
            panic!("find body was {body}");
        }
    }
}
