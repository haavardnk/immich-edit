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
    pub make: Vec<String>,
    #[serde(default)]
    pub model: Vec<String>,
    #[serde(default)]
    pub lens_model: Vec<String>,
    #[serde(default)]
    pub country: Vec<String>,
    #[serde(default)]
    pub state: Vec<String>,
    #[serde(default)]
    pub city: Vec<String>,
    #[serde(skip)]
    pub added_between: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl BrowseFilter {
    pub fn exif_values(&self) -> [(&'static str, &[String]); 6] {
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
