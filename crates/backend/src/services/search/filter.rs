use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::search::ids::IdSet;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowseFilter {
    pub scope: Option<Scope>,
    pub favorite: bool,
    pub not_in_album: bool,
    pub untagged: bool,
    pub no_people: bool,
    pub albums: IdSet,
    pub people: IdSet,
    pub tags: IdSet,
    pub visibility: Option<Visibility>,
    pub images_only: bool,
    pub rating: Option<RatingMatch>,
    pub filename: Option<String>,
    pub taken_after: Option<DateTime<Utc>>,
    pub taken_before: Option<DateTime<Utc>>,
    pub rejected: Option<Rejected>,
    pub label: Option<Label>,
    pub make: ExifMatch,
    pub model: ExifMatch,
    pub lens_model: ExifMatch,
    pub country: ExifMatch,
    pub state: ExifMatch,
    pub city: ExifMatch,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub added_after: Option<DateTime<Utc>>,
    pub added_before: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub added_between: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl BrowseFilter {
    pub fn exif_values(&self) -> [(&'static str, &ExifMatch); 6] {
        [
            ("make", &self.make),
            ("model", &self.model),
            ("lensModel", &self.lens_model),
            ("country", &self.country),
            ("state", &self.state),
            ("city", &self.city),
        ]
    }

    pub fn added_bounds(&self) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        let (low, high) = self.added_between.unzip();
        (
            [self.added_after, low].into_iter().flatten().max(),
            [self.added_before, high].into_iter().flatten().min(),
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<Option<String>>")]
pub enum ExifMatch {
    #[default]
    Any,
    Unknown,
    In(Vec<String>),
}

impl TryFrom<Vec<Option<String>>> for ExifMatch {
    type Error = &'static str;

    fn try_from(values: Vec<Option<String>>) -> Result<Self, Self::Error> {
        match values.as_slice() {
            [] => Ok(Self::Any),
            [None] => Ok(Self::Unknown),
            _ => values
                .into_iter()
                .collect::<Option<Vec<String>>>()
                .map(Self::In)
                .ok_or("an unknown value cannot be combined with other values"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Album(Uuid),
    Person(Uuid),
    Tag(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Timeline,
    Archive,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RatingMatch {
    Eq(i8),
    Gte(i8),
    Lte(i8),
    Unrated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Rejected {
    Hide,
    Only,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Label {
    None,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn picked_ids_read_as_id_sets() {
        let album = Uuid::new_v4();
        let person = Uuid::new_v4();
        let tag = Uuid::new_v4();
        let read = serde_json::from_value::<BrowseFilter>(json!({
            "albums": { "all": [album] },
            "people": { "none": [person] },
            "tags": { "any": [tag] }
        }))
        .ok()
        .map(|filter| (filter.albums, filter.people, filter.tags));
        let expected = (
            IdSet {
                all: vec![album],
                ..IdSet::default()
            },
            IdSet {
                none: vec![person],
                ..IdSet::default()
            },
            IdSet {
                any: vec![tag],
                ..IdSet::default()
            },
        );
        if read != Some(expected) {
            panic!("ids read as {read:?}");
        }
    }

    #[test]
    fn exif_values_read_unknown_only_on_its_own() {
        let cases = [
            (json!([]), Some(ExifMatch::Any)),
            (json!([null]), Some(ExifMatch::Unknown)),
            (
                json!(["SONY", "Canon"]),
                Some(ExifMatch::In(vec!["SONY".into(), "Canon".into()])),
            ),
            (json!([null, "SONY"]), None),
        ];
        for (raw, expected) in cases {
            let read = serde_json::from_value::<BrowseFilter>(json!({ "make": raw.clone() }))
                .ok()
                .map(|filter| filter.make);
            if read != expected {
                panic!("make {raw} read as {read:?}");
            }
        }
    }
}
