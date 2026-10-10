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
    Eq(u8),
    Gte(u8),
    Unrated,
}
