use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::services::search::filter::{BrowseFilter, RatingMatch, Scope};
use crate::services::search::query::{PageQuery, SmartQuery};

pub fn timestamp(at: DateTime<Utc>) -> Value {
    at.to_rfc3339_opts(SecondsFormat::Millis, true).into()
}

pub fn filter(filter: &BrowseFilter) -> Map<String, Value> {
    let mut body = Map::new();
    match filter.scope {
        Some(Scope::Album(id)) => body.insert("albumIds".into(), json!([id])),
        Some(Scope::Person(id)) => body.insert("personIds".into(), json!([id])),
        Some(Scope::Tag(id)) => body.insert("tagIds".into(), json!([id])),
        None => None,
    };
    if filter.favorite {
        body.insert("isFavorite".into(), true.into());
    }
    if let Some(visibility) = filter.visibility {
        body.insert("visibility".into(), json!(visibility));
    }
    if filter.images_only {
        body.insert("type".into(), "IMAGE".into());
    }
    match filter.rating {
        Some(RatingMatch::Eq(stars)) => body.insert("rating".into(), stars.into()),
        Some(RatingMatch::Unrated) => body.insert("rating".into(), Value::Null),
        None => None,
    };
    if let Some(name) = &filter.filename {
        body.insert("originalFileName".into(), name.clone().into());
    }
    if let Some(after) = filter.taken_after {
        body.insert("takenAfter".into(), timestamp(after));
    }
    if let Some(before) = filter.taken_before {
        body.insert("takenBefore".into(), timestamp(before));
    }
    body
}

pub fn page(query: &PageQuery, page: u32) -> Value {
    let mut body = filter(&query.filter);
    body.insert("order".into(), json!(query.sort.direction));
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Value::Object(body)
}

pub fn smart(query: &SmartQuery, page: u32) -> Value {
    let mut body = filter(&query.filter);
    body.insert("query".into(), query.text.clone().into());
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Value::Object(body)
}

pub fn find(filter_by: &BrowseFilter, id: Uuid) -> Value {
    let mut body = filter(filter_by);
    body.insert("id".into(), id.to_string().into());
    body.insert("size".into(), 1.into());
    body.insert("withExif".into(), true.into());
    Value::Object(body)
}

pub fn count(filter_by: &BrowseFilter) -> Value {
    Value::Object(filter(filter_by))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::services::search::filter::Visibility;
    use crate::services::search::query::{Direction, Sort, SortField};

    #[test]
    fn a_browse_page_maps_to_flat_immich_fields() {
        let tag = Uuid::new_v4();
        let query = PageQuery {
            filter: BrowseFilter {
                scope: Some(Scope::Tag(tag)),
                favorite: true,
                visibility: Some(Visibility::Archive),
                images_only: true,
                rating: Some(RatingMatch::Unrated),
                filename: Some("DSC".into()),
                taken_after: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
                taken_before: None,
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
            "tagIds": [tag],
            "isFavorite": true,
            "visibility": "archive",
            "type": "IMAGE",
            "rating": null,
            "originalFileName": "DSC",
            "takenAfter": "2024-01-01T00:00:00.000Z",
            "order": "asc",
            "size": 500,
            "page": 2,
            "withExif": true,
        });
        if page(&query, 2) != expected {
            panic!("flat body was {}", page(&query, 2));
        }
    }
}
