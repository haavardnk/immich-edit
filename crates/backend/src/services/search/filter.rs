use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowseFilter {
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub not_in_album: bool,
    #[serde(default)]
    pub untagged: bool,
    #[serde(default)]
    pub no_people: bool,
    #[serde(default)]
    pub visibility: Option<Visibility>,
    #[serde(default)]
    pub images_only: bool,
    #[serde(default)]
    pub rating: Option<RatingMatch>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub taken_after: Option<DateTime<Utc>>,
    #[serde(default)]
    pub taken_before: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rejected: Option<Rejected>,
    #[serde(default)]
    pub label: Option<Label>,
    #[serde(default)]
    pub make: ExifMatch,
    #[serde(default)]
    pub model: ExifMatch,
    #[serde(default)]
    pub lens_model: ExifMatch,
    #[serde(default)]
    pub country: ExifMatch,
    #[serde(default)]
    pub state: ExifMatch,
    #[serde(default)]
    pub city: ExifMatch,
    #[serde(default)]
    pub min_size: Option<u64>,
    #[serde(default)]
    pub max_size: Option<u64>,
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
