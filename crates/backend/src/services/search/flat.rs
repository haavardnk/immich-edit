use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::search::filter::{BrowseFilter, RatingMatch, Scope};
use crate::services::search::query::{PageQuery, SmartQuery};
use crate::services::search::tags::TagSet;

pub fn timestamp(at: DateTime<Utc>) -> Value {
    at.to_rfc3339_opts(SecondsFormat::Millis, true).into()
}

pub fn filter(filter: &BrowseFilter, tags: &TagSet) -> Result<Map<String, Value>, AppError> {
    let mut body = Map::new();
    match filter.scope {
        Some(Scope::Album(id)) => body.insert("albumIds".into(), json!([id])),
        Some(Scope::Person(id)) => body.insert("personIds".into(), json!([id])),
        Some(Scope::Tag(_)) | None => None,
    };
    if !tags.none.is_empty() {
        return Err(AppError::BadRequest(
            "this search cannot exclude tagged photos".into(),
        ));
    }
    if !tags.all.is_empty() {
        body.insert("tagIds".into(), json!(tags.all));
    }
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
        Some(RatingMatch::Gte(_) | RatingMatch::Lte(_)) => {
            return Err(AppError::BadRequest(
                "this search cannot filter by a rating range".into(),
            ));
        }
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
    if let Some((after, before)) = filter.added_between {
        body.insert("createdAfter".into(), timestamp(after));
        body.insert("createdBefore".into(), timestamp(before));
    }
    Ok(body)
}

pub fn page(query: &PageQuery, tags: &TagSet, page: u32) -> Result<Value, AppError> {
    let mut body = filter(&query.filter, tags)?;
    body.insert("order".into(), json!(query.sort.direction));
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Ok(Value::Object(body))
}

pub fn smart(query: &SmartQuery, tags: &TagSet, page: u32) -> Result<Value, AppError> {
    let mut body = filter(&query.filter, tags)?;
    body.insert("query".into(), query.text.clone().into());
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Ok(Value::Object(body))
}

pub fn find(filter_by: &BrowseFilter, tags: &TagSet, id: Uuid) -> Result<Value, AppError> {
    let mut body = filter(filter_by, tags)?;
    body.insert("id".into(), id.to_string().into());
    body.insert("size".into(), 1.into());
    body.insert("withExif".into(), true.into());
    Ok(Value::Object(body))
}

pub fn count(filter_by: &BrowseFilter, tags: &TagSet) -> Result<Value, AppError> {
    Ok(Value::Object(filter(filter_by, tags)?))
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
        let reject = Uuid::new_v4();
        let tags = TagSet {
            all: vec![tag, reject],
            none: vec![],
        };
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
            "tagIds": [tag, reject],
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
        if page(&query, &tags, 2).ok() != Some(expected) {
            panic!("flat body was {:?}", page(&query, &tags, 2));
        }
    }

    #[test]
    fn flat_servers_refuse_what_they_cannot_express() {
        let minimum = BrowseFilter {
            rating: Some(RatingMatch::Gte(3)),
            ..BrowseFilter::default()
        };
        let maximum = BrowseFilter {
            rating: Some(RatingMatch::Lte(2)),
            ..BrowseFilter::default()
        };
        let excluded = TagSet {
            all: vec![],
            none: vec![Uuid::new_v4()],
        };
        let cases = [
            (minimum, TagSet::default()),
            (maximum, TagSet::default()),
            (BrowseFilter::default(), excluded),
        ];
        for (filter_by, tags) in cases {
            if count(&filter_by, &tags).is_ok() {
                panic!("flat search accepted {filter_by:?} {tags:?}");
            }
        }
    }
}
