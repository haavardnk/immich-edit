use std::collections::BTreeSet;

use futures_util::future::try_join_all;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::names::display_name;
use crate::services::edits_store::{ExportOutputQuery, RecordedExport};
use crate::state::AppState;

use super::conflict::{Previous, previous_export};
use super::naming::preset_key;
use super::variants::MAX_VARIANTS;

const MAX_ASSETS: usize = 10_000;

#[derive(Debug, Deserialize)]
pub struct ExportOutputsQuery {
    pub asset_ids: Vec<String>,
    #[serde(default)]
    pub presets: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutputLookup {
    asset_ids: Vec<AssetKey>,
    presets: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ExportedFile {
    pub filename: String,
    pub exported_at: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ExportOutputsSummary {
    pub photos: u64,
    pub files: Vec<ExportedFile>,
}

impl ExportOutputsQuery {
    pub fn parse(&self) -> Result<OutputLookup, AppError> {
        if self.asset_ids.is_empty() || self.asset_ids.len() > MAX_ASSETS {
            return Err(AppError::BadRequest(format!(
                "a lookup takes 1 to {MAX_ASSETS} photos"
            )));
        }
        if self.presets.len() > MAX_VARIANTS {
            return Err(AppError::BadRequest(format!(
                "a lookup takes at most {MAX_VARIANTS} presets"
            )));
        }
        let asset_ids = self
            .asset_ids
            .iter()
            .map(|id| {
                id.parse::<AssetKey>()
                    .map_err(|_| AppError::BadRequest(format!("invalid asset id: {id}")))
            })
            .collect::<Result<BTreeSet<AssetKey>, AppError>>()?;
        let mut presets = self
            .presets
            .iter()
            .map(|name| {
                display_name(name, "preset name")
                    .map(|name| preset_key(&name))
                    .map_err(AppError::BadRequest)
            })
            .collect::<Result<BTreeSet<String>, AppError>>()?;
        if presets.is_empty() {
            presets.insert(String::new());
        }
        Ok(OutputLookup {
            asset_ids: asset_ids.into_iter().collect(),
            presets: presets.into_iter().collect(),
        })
    }
}

pub async fn find_export_outputs(
    state: &AppState,
    immich: &ImmichClient,
    owner: Uuid,
    lookup: &OutputLookup,
) -> Result<ExportOutputsSummary, AppError> {
    let query = ExportOutputQuery {
        owner,
        asset_ids: &lookup.asset_ids,
        presets: &lookup.presets,
    };
    if lookup.asset_ids.len() > 1 {
        return Ok(ExportOutputsSummary {
            photos: state.edits.count_exported_assets(query).await?,
            files: Vec::new(),
        });
    }
    let recorded = state.edits.recorded_exports(query).await?;
    let checked = try_join_all(recorded.into_iter().map(|r| live_file(immich, r))).await?;
    let files: Vec<ExportedFile> = checked.into_iter().flatten().collect();
    Ok(ExportOutputsSummary {
        photos: u64::from(!files.is_empty()),
        files,
    })
}

async fn live_file(
    immich: &ImmichClient,
    recorded: RecordedExport,
) -> Result<Option<ExportedFile>, AppError> {
    Ok(
        match previous_export(immich, recorded.immich_asset_id).await? {
            Previous::Live(filename) => Some(ExportedFile {
                filename,
                exported_at: recorded.exported_at,
            }),
            Previous::Trashed | Previous::Gone => None,
        },
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn lookup(body: Value) -> Result<OutputLookup, AppError> {
        serde_json::from_value::<ExportOutputsQuery>(body)
            .unwrap()
            .parse()
    }

    fn rejection(body: Value) -> String {
        match lookup(body) {
            Err(AppError::BadRequest(message)) => message,
            other => panic!("expected a 400, got {other:?}"),
        }
    }

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|_| Uuid::new_v4().to_string()).collect()
    }

    #[test]
    fn ids_are_parsed_and_deduplicated() {
        let source = Uuid::new_v4();
        let copy = format!("{source}_2");

        let parsed = lookup(json!({
            "asset_ids": [source.to_string(), copy, source.to_string().to_uppercase()]
        }))
        .unwrap();

        assert_eq!(
            parsed.asset_ids,
            [AssetKey::master(source), copy.parse().unwrap()]
        );
    }

    #[test]
    fn no_presets_means_exports_without_a_preset() {
        let parsed = lookup(json!({ "asset_ids": ids(1) })).unwrap();
        let empty = lookup(json!({ "asset_ids": ids(1), "presets": [] })).unwrap();

        assert_eq!(parsed.presets, [""]);
        assert_eq!(empty.presets, [""]);
    }

    #[test]
    fn presets_are_keyed_like_export_outputs() {
        let parsed = lookup(json!({
            "asset_ids": ids(1),
            "presets": [" Web 2048 ", "web 2048", "A/B", "a:b"]
        }))
        .unwrap();

        assert_eq!(parsed.presets, ["a_b", "web 2048"]);
    }

    #[test]
    fn a_lookup_takes_one_to_ten_thousand_photos() {
        for n in [0, 10_001] {
            assert_eq!(
                rejection(json!({ "asset_ids": ids(n) })),
                "a lookup takes 1 to 10000 photos",
                "{n}"
            );
        }
        assert_eq!(
            lookup(json!({ "asset_ids": ids(10_000) }))
                .unwrap()
                .asset_ids
                .len(),
            10_000
        );
    }

    #[test]
    fn a_lookup_takes_at_most_sixteen_presets() {
        let names = |n: usize| -> Vec<String> { (0..n).map(|i| format!("P{i}")).collect() };

        assert_eq!(
            rejection(json!({ "asset_ids": ids(1), "presets": names(17) })),
            "a lookup takes at most 16 presets"
        );
        assert_eq!(
            lookup(json!({ "asset_ids": ids(1), "presets": names(16) }))
                .unwrap()
                .presets
                .len(),
            16
        );
    }

    #[test]
    fn bad_ids_and_preset_names_are_rejected() {
        for (body, message) in [
            (
                json!({ "asset_ids": ["nope"] }),
                "invalid asset id: nope".to_string(),
            ),
            (
                json!({ "asset_ids": [format!("{}_0", Uuid::nil())] }),
                format!("invalid asset id: {}_0", Uuid::nil()),
            ),
            (
                json!({ "asset_ids": ids(1), "presets": ["  "] }),
                "preset name required".to_string(),
            ),
        ] {
            assert_eq!(rejection(body), message);
        }
    }
}
