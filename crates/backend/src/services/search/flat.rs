use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::search::filter::{BrowseFilter, ExifMatch, RatingMatch, TextField};
use crate::services::search::ids::Ids;
use crate::services::search::query::{PageQuery, SmartQuery};

pub fn timestamp(at: DateTime<Utc>) -> Value {
    at.to_rfc3339_opts(SecondsFormat::Millis, true).into()
}

pub fn filter(filter: &BrowseFilter, ids: &Ids) -> Result<Map<String, Value>, AppError> {
    let mut body = Map::new();
    for (field, set) in ids.fields() {
        if !set.none.is_empty() || set.any.len() > 1 {
            return Err(AppError::BadRequest(format!(
                "this search can only match photos with all of the {field}"
            )));
        }
        let required: Vec<Uuid> = set.all.iter().chain(&set.any).copied().collect();
        if !required.is_empty() {
            body.insert(field.into(), json!(required));
        }
    }
    if filter.not_in_album {
        body.insert("isNotInAlbum".into(), true.into());
    }
    if filter.untagged {
        body.insert("tagIds".into(), Value::Null);
    }
    if filter.no_people {
        return Err(AppError::BadRequest(
            "this search cannot find photos without people".into(),
        ));
    }
    if filter.min_size.is_some() || filter.max_size.is_some() {
        return Err(AppError::BadRequest(
            "this search cannot filter by file size".into(),
        ));
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
    for (field, text) in filter.text_matches() {
        body.insert(field.immich_field().into(), text.into());
    }
    if let Some(after) = filter.taken_after {
        body.insert("takenAfter".into(), timestamp(after));
    }
    if let Some(before) = filter.taken_before {
        body.insert("takenBefore".into(), timestamp(before));
    }
    for (field, values) in filter.exif_values() {
        match values {
            ExifMatch::Any => {}
            ExifMatch::Unknown => {
                body.insert(field.into(), Value::Null);
            }
            ExifMatch::In(values) if values.len() == 1 => {
                body.insert(field.into(), values[0].as_str().into());
            }
            ExifMatch::In(_) => {
                return Err(AppError::BadRequest(format!(
                    "this search cannot match several {field} values"
                )));
            }
        }
    }
    let (added_after, added_before) = filter.added_bounds();
    if let Some(after) = added_after {
        body.insert("createdAfter".into(), timestamp(after));
    }
    if let Some(before) = added_before {
        body.insert("createdBefore".into(), timestamp(before));
    }
    Ok(body)
}

pub fn page(query: &PageQuery, ids: &Ids, page: u32) -> Result<Value, AppError> {
    let mut body = filter(&query.filter, ids)?;
    body.insert("order".into(), json!(query.sort.direction));
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Ok(Value::Object(body))
}

pub fn smart(query: &SmartQuery, ids: &Ids, page: u32) -> Result<Value, AppError> {
    if let Some((field, _)) = query
        .filter
        .text_matches()
        .find(|(field, _)| *field != TextField::Ocr)
    {
        return Err(AppError::BadRequest(format!(
            "smart search cannot filter by {}",
            field.immich_field()
        )));
    }
    let mut body = filter(&query.filter, ids)?;
    body.insert("query".into(), query.text.clone().into());
    body.insert("size".into(), query.size.into());
    body.insert("page".into(), page.into());
    if query.with_exif {
        body.insert("withExif".into(), true.into());
    }
    Ok(Value::Object(body))
}

pub fn find(filter_by: &BrowseFilter, ids: &Ids, id: Uuid) -> Result<Value, AppError> {
    let mut body = filter(filter_by, ids)?;
    body.insert("id".into(), id.to_string().into());
    body.insert("size".into(), 1.into());
    body.insert("withExif".into(), true.into());
    Ok(Value::Object(body))
}

pub fn count(filter_by: &BrowseFilter, ids: &Ids) -> Result<Value, AppError> {
    Ok(Value::Object(filter(filter_by, ids)?))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::services::search::filter::Visibility;
    use crate::services::search::ids::IdSet;
    use crate::services::search::query::{Direction, Sort, SortField};

    #[test]
    fn a_browse_page_maps_to_flat_immich_fields() {
        let tag = Uuid::new_v4();
        let reject = Uuid::new_v4();
        let person = Uuid::new_v4();
        let ids = Ids {
            people: IdSet {
                any: vec![person],
                ..IdSet::default()
            },
            tags: IdSet {
                all: vec![tag, reject],
                ..IdSet::default()
            },
            ..Ids::default()
        };
        let query = PageQuery {
            filter: BrowseFilter {
                favorite: true,
                not_in_album: true,
                visibility: Some(Visibility::Archive),
                images_only: true,
                rating: Some(RatingMatch::Unrated),
                filename: Some("DSC".into()),
                taken_after: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
                make: ExifMatch::In(vec!["SONY".into()]),
                lens_model: ExifMatch::In(vec!["FE 35mm F1.8".into()]),
                country: ExifMatch::Unknown,
                city: ExifMatch::In(vec!["Bergen".into()]),
                added_before: Some(Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()),
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
            "isNotInAlbum": true,
            "personIds": [person],
            "tagIds": [tag, reject],
            "isFavorite": true,
            "visibility": "archive",
            "type": "IMAGE",
            "rating": null,
            "originalFileName": "DSC",
            "takenAfter": "2024-01-01T00:00:00.000Z",
            "make": "SONY",
            "lensModel": "FE 35mm F1.8",
            "country": null,
            "city": "Bergen",
            "createdBefore": "2025-06-01T00:00:00.000Z",
            "order": "asc",
            "size": 500,
            "page": 2,
            "withExif": true,
        });
        if page(&query, &ids, 2).ok() != Some(expected) {
            panic!("flat body was {:?}", page(&query, &ids, 2));
        }
    }

    #[test]
    fn untagged_asks_for_a_null_tag_list() {
        let filter_by = BrowseFilter {
            untagged: true,
            ..BrowseFilter::default()
        };
        let body = count(&filter_by, &Ids::default()).ok();
        if body != Some(json!({ "tagIds": null })) {
            panic!("untagged flat body was {body:?}");
        }
    }

    #[test]
    fn text_fields_map_to_their_flat_immich_fields() {
        let filter_by = BrowseFilter {
            description: Some(" fjord ".into()),
            path: Some("/2024/".into()),
            ocr: Some("exit".into()),
            filename: Some("  ".into()),
            ..BrowseFilter::default()
        };
        let expected = json!({ "description": "fjord", "originalPath": "/2024/", "ocr": "exit" });
        let body = count(&filter_by, &Ids::default()).ok();
        if body != Some(expected) {
            panic!("text flat body was {body:?}");
        }
    }

    #[test]
    fn smart_search_refuses_text_fields_immich_would_drop() {
        let text = |field: fn(&mut BrowseFilter) -> &mut Option<String>| {
            let mut filter_by = BrowseFilter::default();
            *field(&mut filter_by) = Some("x".into());
            filter_by
        };
        let cases = [
            (text(|f| &mut f.filename), false),
            (text(|f| &mut f.description), false),
            (text(|f| &mut f.path), false),
            (text(|f| &mut f.ocr), true),
        ];
        for (filter_by, accepted) in cases {
            let query = SmartQuery {
                text: "red car".into(),
                filter: filter_by,
                size: 10,
                cursor: None,
                with_exif: false,
            };
            if smart(&query, &Ids::default(), 1).is_ok() != accepted {
                panic!("smart search with {:?}", query.filter);
            }
        }
    }

    #[test]
    fn flat_servers_refuse_what_they_cannot_express() {
        let filters = [
            BrowseFilter {
                rating: Some(RatingMatch::Gte(3)),
                ..BrowseFilter::default()
            },
            BrowseFilter {
                rating: Some(RatingMatch::Lte(2)),
                ..BrowseFilter::default()
            },
            BrowseFilter {
                model: ExifMatch::In(vec!["ILCE-7M3".into(), "X-T5".into()]),
                ..BrowseFilter::default()
            },
            BrowseFilter {
                no_people: true,
                ..BrowseFilter::default()
            },
            BrowseFilter {
                max_size: Some(5_000_000),
                ..BrowseFilter::default()
            },
        ];
        let id_sets = [
            Ids {
                tags: IdSet {
                    none: vec![Uuid::new_v4()],
                    ..IdSet::default()
                },
                ..Ids::default()
            },
            Ids {
                albums: IdSet {
                    any: vec![Uuid::new_v4(), Uuid::new_v4()],
                    ..IdSet::default()
                },
                ..Ids::default()
            },
        ];
        let cases = filters
            .into_iter()
            .map(|filter_by| (filter_by, Ids::default()))
            .chain(
                id_sets
                    .into_iter()
                    .map(|ids| (BrowseFilter::default(), ids)),
            );
        for (filter_by, ids) in cases {
            if count(&filter_by, &ids).is_ok() {
                panic!("flat search accepted {filter_by:?} {ids:?}");
            }
        }
    }
}
