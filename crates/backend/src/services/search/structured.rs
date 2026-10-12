use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::services::search::filter::{BrowseFilter, ExifMatch, RatingMatch, TextField};
use crate::services::search::flat::timestamp;
use crate::services::search::ids::Ids;
use crate::services::search::query::{PageQuery, SortField};

pub fn filter(filter: &BrowseFilter, ids: &Ids) -> Map<String, Value> {
    let mut body = Map::new();
    body.insert("trashedAt".into(), json!({ "eq": null }));
    for (field, set) in ids.fields() {
        let matched: Map<String, Value> =
            [("all", &set.all), ("any", &set.any), ("none", &set.none)]
                .into_iter()
                .filter(|(_, ids)| !ids.is_empty())
                .map(|(key, ids)| (key.into(), json!(ids)))
                .collect();
        if !matched.is_empty() {
            body.insert(field.into(), Value::Object(matched));
        }
    }
    if filter.not_in_album {
        body.insert("hasAlbums".into(), json!({ "eq": false }));
    }
    if filter.untagged {
        body.insert("hasTags".into(), json!({ "eq": false }));
    }
    if filter.no_people {
        body.insert("hasPeople".into(), json!({ "eq": false }));
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
    for (field, text) in filter.text_matches() {
        let matched = match field {
            TextField::Ocr => json!({ "matches": text }),
            TextField::FileName | TextField::Description | TextField::Path => {
                json!({ "like": text })
            }
        };
        body.insert(field.immich_field().into(), matched);
    }
    range(
        &mut body,
        "takenAt",
        filter.taken_after.map(timestamp),
        filter.taken_before.map(timestamp),
    );
    range(
        &mut body,
        "fileSizeInBytes",
        filter.min_size.map(Value::from),
        filter.max_size.map(Value::from),
    );
    for (field, values) in filter.exif_values() {
        match values {
            ExifMatch::Any => {}
            ExifMatch::Unknown => {
                body.insert(field.into(), json!({ "eq": null }));
            }
            ExifMatch::In(values) => {
                body.insert(field.into(), json!({ "in": values }));
            }
        }
    }
    let (added_after, added_before) = filter.added_bounds();
    range(
        &mut body,
        "createdAt",
        added_after.map(timestamp),
        added_before.map(timestamp),
    );
    body
}

fn range(body: &mut Map<String, Value>, field: &str, gte: Option<Value>, lte: Option<Value>) {
    let bounds: Map<String, Value> = [("gte", gte), ("lte", lte)]
        .into_iter()
        .filter_map(|(key, bound)| Some((key.into(), bound?)))
        .collect();
    if !bounds.is_empty() {
        body.insert(field.into(), Value::Object(bounds));
    }
}

pub fn page(query: &PageQuery, ids: &Ids, cursor: Option<&str>) -> Value {
    let field = match query.sort.field {
        SortField::Rating => "rating",
        SortField::Size => "fileSizeInBytes",
        SortField::Taken | SortField::Added => "fileCreatedAt",
    };
    let mut body = json!({
        "filter": filter(&query.filter, ids),
        "orderBy": { "field": field, "direction": query.sort.direction },
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

pub fn find(filter_by: &BrowseFilter, ids: &Ids, id: Uuid) -> Value {
    let mut conditions = filter(filter_by, ids);
    conditions.insert("id".into(), json!({ "eq": id }));
    json!({ "filter": conditions, "size": 1, "withExif": true })
}

pub fn count(filter_by: &BrowseFilter, ids: &Ids) -> Value {
    json!({ "filter": filter(filter_by, ids) })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::services::search::filter::Visibility;
    use crate::services::search::ids::IdSet;
    use crate::services::search::query::{Direction, Sort, SortField};

    #[test]
    fn a_browse_page_maps_to_a_structured_immich_filter() {
        let album = Uuid::new_v4();
        let people = IdSet {
            any: vec![Uuid::new_v4(), Uuid::new_v4()],
            ..IdSet::default()
        };
        let tags = IdSet {
            all: vec![Uuid::new_v4()],
            none: vec![Uuid::new_v4()],
            ..IdSet::default()
        };
        let ids = Ids {
            albums: IdSet {
                all: vec![album],
                ..IdSet::default()
            },
            people: people.clone(),
            tags: tags.clone(),
        };
        let query = PageQuery {
            filter: BrowseFilter {
                favorite: true,
                not_in_album: true,
                untagged: true,
                no_people: true,
                visibility: Some(Visibility::Timeline),
                images_only: true,
                rating: Some(RatingMatch::Eq(3)),
                filename: Some(" DSC ".into()),
                taken_before: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
                make: ExifMatch::In(vec!["SONY".into(), "FUJIFILM".into()]),
                lens_model: ExifMatch::Unknown,
                state: ExifMatch::In(vec!["Vestland".into(), "Oslo".into()]),
                min_size: Some(1_000_000),
                added_after: Some(Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()),
                ..BrowseFilter::default()
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
                "personIds": { "any": people.any },
                "hasAlbums": { "eq": false },
                "hasTags": { "eq": false },
                "hasPeople": { "eq": false },
                "tagIds": { "all": tags.all, "none": tags.none },
                "isFavorite": { "eq": true },
                "visibility": { "eq": "timeline" },
                "type": { "eq": "IMAGE" },
                "rating": { "eq": 3 },
                "originalFileName": { "like": "DSC" },
                "takenAt": { "lte": "2024-01-01T00:00:00.000Z" },
                "make": { "in": ["SONY", "FUJIFILM"] },
                "lensModel": { "eq": null },
                "state": { "in": ["Vestland", "Oslo"] },
                "fileSizeInBytes": { "gte": 1_000_000 },
                "createdAt": { "gte": "2025-06-01T00:00:00.000Z" },
            },
            "orderBy": { "field": "fileCreatedAt", "direction": "asc" },
            "size": 500,
            "cursor": "next",
            "withExif": true,
        });
        let body = page(&query, &ids, Some("next"));
        if body != expected {
            panic!("structured body was {body}");
        }
    }

    #[test]
    fn text_fields_match_substrings_and_ocr_by_similarity() {
        let filter_by = BrowseFilter {
            description: Some(" fjord ".into()),
            path: Some("/2024/".into()),
            ocr: Some("exit".into()),
            filename: Some("  ".into()),
            ..BrowseFilter::default()
        };
        let expected = json!({
            "filter": {
                "trashedAt": { "eq": null },
                "description": { "like": "fjord" },
                "originalPath": { "like": "/2024/" },
                "ocr": { "matches": "exit" },
            }
        });
        let body = count(&filter_by, &Ids::default());
        if body != expected {
            panic!("text body was {body}");
        }
    }

    #[test]
    fn a_rating_sort_orders_by_rating() {
        let query = PageQuery {
            filter: BrowseFilter::default(),
            sort: Sort {
                field: SortField::Rating,
                direction: Direction::Desc,
            },
            size: 10,
            cursor: None,
            with_exif: false,
        };
        let body = page(&query, &Ids::default(), None);
        if body["orderBy"] != json!({ "field": "rating", "direction": "desc" }) {
            panic!("rating sort body was {body}");
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
        let body = find(&filter_by, &Ids::default(), id);
        if body != expected {
            panic!("find body was {body}");
        }
    }
}
