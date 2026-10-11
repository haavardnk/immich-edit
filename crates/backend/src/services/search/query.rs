use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::search::filter::BrowseFilter;

pub const MAX_SIZE: u32 = 1000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    #[default]
    Taken,
    Added,
    Rating,
    Size,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sort {
    #[serde(default)]
    pub field: SortField,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageQuery {
    #[serde(default)]
    pub filter: BrowseFilter,
    #[serde(default)]
    pub sort: Sort,
    pub size: u32,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub with_exif: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SmartQuery {
    pub text: String,
    #[serde(default)]
    pub filter: BrowseFilter,
    pub size: u32,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub with_exif: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CountQuery {
    #[serde(default)]
    pub filter: BrowseFilter,
}

pub fn check_size(size: u32) -> Result<(), AppError> {
    if (1..=MAX_SIZE).contains(&size) {
        return Ok(());
    }
    Err(AppError::BadRequest(format!(
        "size must be 1 to {MAX_SIZE}"
    )))
}
