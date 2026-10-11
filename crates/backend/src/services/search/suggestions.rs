use futures_util::future::try_join_all;
use serde::Deserialize;

use crate::error::AppError;
use crate::immich::ImmichClient;

pub const MAX_NARROWING: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Field {
    Make,
    Model,
    LensModel,
    Country,
    State,
    City,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SuggestionQuery {
    pub field: Field,
    #[serde(default)]
    pub make: Vec<String>,
    #[serde(default)]
    pub model: Vec<String>,
    #[serde(default)]
    pub country: Vec<String>,
    #[serde(default)]
    pub state: Vec<String>,
}

impl SuggestionQuery {
    fn immich_type(&self) -> &'static str {
        match self.field {
            Field::Make => "camera-make",
            Field::Model => "camera-model",
            Field::LensModel => "camera-lens-model",
            Field::Country => "country",
            Field::State => "state",
            Field::City => "city",
        }
    }

    fn narrowing(&self) -> Option<(&'static str, &[String])> {
        let parents = match self.field {
            Field::Make | Field::Country => vec![],
            Field::Model => vec![("make", self.make.as_slice())],
            Field::LensModel => vec![
                ("model", self.model.as_slice()),
                ("make", self.make.as_slice()),
            ],
            Field::State => vec![("country", self.country.as_slice())],
            Field::City => vec![
                ("state", self.state.as_slice()),
                ("country", self.country.as_slice()),
            ],
        };
        parents.into_iter().find(|(_, values)| !values.is_empty())
    }
}

pub async fn values(
    immich: &ImmichClient,
    query: &SuggestionQuery,
) -> Result<Vec<Option<String>>, AppError> {
    let kind = query.immich_type();
    let requests: Vec<Vec<(&str, &str)>> = match query.narrowing() {
        Some((param, parents)) if parents.len() <= MAX_NARROWING => parents
            .iter()
            .map(|parent| {
                vec![
                    ("type", kind),
                    ("includeNull", "true"),
                    (param, parent.as_str()),
                ]
            })
            .collect(),
        _ => vec![vec![("type", kind), ("includeNull", "true")]],
    };
    let lists = try_join_all(
        requests
            .iter()
            .map(|params| immich.search_suggestions(params)),
    )
    .await?;
    let mut values: Vec<Option<String>> = lists.into_iter().flatten().collect();
    values.sort_by(|a, b| match (a, b) {
        (Some(a), Some(b)) => a
            .to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b)),
        _ => a.is_none().cmp(&b.is_none()),
    });
    values.dedup();
    Ok(values)
}
