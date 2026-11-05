use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::names::display_name;
use crate::services::job_runner::JobItemError;
use crate::services::job_store::{JobItemRecord, JobRecord, NewJobItem};

use super::batch::{ExportJobParams, validate_export_job};
use super::naming::preset_key;
use super::{DOWNLOAD_ZIP_KIND, EXPORT_JOB_KIND, Seq};

const MAX_VARIANTS: usize = 16;
const VARIANTS_KEY: &str = "variants";
const PRESET_KEY: &str = "preset";
const VARIANT_TEMPLATE: &str = "{name}_{preset}";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVariant {
    preset: String,
    params: Value,
}

#[derive(Debug)]
struct ExportVariant {
    preset: Option<String>,
    params: ExportJobParams,
}

#[derive(Debug)]
pub struct ExportVariants(Vec<ExportVariant>);

pub(super) struct ItemExport {
    pub(super) preset: Option<String>,
    pub(super) params: ExportJobParams,
    pub(super) seq: Seq,
}

fn variants_entry(params: &Value) -> Option<(&Map<String, Value>, &Value)> {
    let object = params.as_object()?;
    Some((object, object.get(VARIANTS_KEY)?))
}

fn variant_params(preset: &str, raw: &Value) -> Result<ExportJobParams, String> {
    let mut params = ExportJobParams::parse(raw).map_err(|e| format!("{preset}: {e}"))?;
    let template = &mut params.params.filename_template;
    if template.as_deref().is_none_or(|t| t.trim().is_empty()) {
        *template = Some(VARIANT_TEMPLATE.into());
    }
    Ok(params)
}

fn file_name_clash(seen: &HashMap<String, String>, preset: &str) -> Option<String> {
    let earlier = seen.get(&preset_key(preset))?;
    Some(if earlier.to_lowercase() == preset.to_lowercase() {
        format!("preset {preset} is listed twice")
    } else {
        format!("presets {earlier} and {preset} give the same file name")
    })
}

impl ExportVariants {
    pub fn parse(params: &Value) -> Result<Self, String> {
        let Some((object, list)) = variants_entry(params) else {
            let single = ExportJobParams::parse(params).map_err(|e| e.to_string())?;
            return Ok(Self(vec![ExportVariant {
                preset: None,
                params: single,
            }]));
        };
        if let Some(extra) = object.keys().find(|key| key.as_str() != VARIANTS_KEY) {
            return Err(format!("unexpected key beside variants: {extra}"));
        }
        let raw =
            Vec::<RawVariant>::deserialize(list).map_err(|e| format!("invalid variants: {e}"))?;
        if raw.is_empty() || raw.len() > MAX_VARIANTS {
            return Err(format!("a job exports 1 to {MAX_VARIANTS} presets"));
        }
        let mut seen = HashMap::new();
        let mut variants = Vec::with_capacity(raw.len());
        for variant in raw {
            let preset = display_name(&variant.preset, "preset name")?;
            if let Some(clash) = file_name_clash(&seen, &preset) {
                return Err(clash);
            }
            seen.insert(preset_key(&preset), preset.clone());
            variants.push(ExportVariant {
                params: variant_params(&preset, &variant.params)?,
                preset: Some(preset),
            });
        }
        Ok(Self(variants))
    }

    pub fn for_job(kind: &str, params: &Value) -> Result<Option<Self>, AppError> {
        let checked = kind == EXPORT_JOB_KIND
            || (kind == DOWNLOAD_ZIP_KIND && variants_entry(params).is_some());
        if !checked {
            return Ok(None);
        }
        Self::parse(params).map(Some).map_err(AppError::BadRequest)
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    fn one_version_each(&self) -> Result<(), AppError> {
        if self.0.len() > 1 && self.0.iter().any(|v| v.params.replace_original) {
            return Err(AppError::BadRequest(
                "replace original exports one version of each photo".into(),
            ));
        }
        Ok(())
    }

    pub async fn validate(
        &self,
        kind: &str,
        immich: &ImmichClient,
        asset_ids: &[String],
    ) -> Result<(), AppError> {
        if kind != EXPORT_JOB_KIND {
            return Ok(());
        }
        self.one_version_each()?;
        for variant in &self.0 {
            validate_export_job(immich, &variant.params, asset_ids).await?;
        }
        Ok(())
    }

    pub async fn validate_items(
        &self,
        kind: &str,
        immich: &ImmichClient,
        items: &[JobItemRecord],
    ) -> Result<(), AppError> {
        if kind != EXPORT_JOB_KIND {
            return Ok(());
        }
        self.one_version_each()?;
        for (index, variant) in (0_i64..).zip(&self.0) {
            let asset_ids: Vec<String> = items
                .iter()
                .filter(|item| item.variant == index)
                .map(|item| item.asset_id.clone())
                .collect();
            if !asset_ids.is_empty() {
                validate_export_job(immich, &variant.params, &asset_ids).await?;
            }
        }
        Ok(())
    }
}

pub fn expand_items(asset_ids: Vec<String>, variants: usize) -> Vec<NewJobItem> {
    (1_i64..)
        .zip(asset_ids)
        .flat_map(|(position, asset_id)| {
            (0_i64..).take(variants).map(move |variant| NewJobItem {
                asset_id: asset_id.clone(),
                idempotency_key: None,
                position,
                variant,
            })
        })
        .collect()
}

fn seq_part(value: i64) -> u32 {
    u32::try_from(value).unwrap_or(1).max(1)
}

pub(super) fn item_export(
    job: &JobRecord,
    item: &JobItemRecord,
) -> Result<ItemExport, JobItemError> {
    let variants = ExportVariants::parse(&job.params).map_err(JobItemError::msg)?;
    let assets = job.total / (variants.count() as i64).max(1);
    let seq = Seq {
        position: seq_part(item.position),
        total: seq_part(assets),
    };
    let variant = usize::try_from(item.variant)
        .ok()
        .and_then(|index| variants.0.into_iter().nth(index))
        .ok_or_else(|| JobItemError::msg(format!("job has no variant {}", item.variant)))?;
    Ok(ItemExport {
        preset: variant.preset,
        params: variant.params,
        seq,
    })
}

pub(super) fn idempotency_key(job_id: Uuid, variant: i64) -> String {
    if variant == 0 {
        format!("job-{job_id}")
    } else {
        format!("job-{job_id}-v{variant}")
    }
}

pub(super) fn tag_preset(mut result: Value, preset: Option<&str>) -> Value {
    if let (Some(preset), Some(object)) = (preset, result.as_object_mut()) {
        object.insert(PRESET_KEY.into(), Value::from(preset));
    }
    result
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::services::export::{NameContext, NameTemplate};
    use crate::services::job_store::{JobItemStatus, JobStatus};

    fn variant(preset: &str, params: Value) -> Value {
        json!({ "preset": preset, "params": params })
    }

    fn rejected(params: Value) -> String {
        ExportVariants::parse(&params).unwrap_err()
    }

    fn job(params: Value, total: i64) -> JobRecord {
        JobRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            server_epoch: 1,
            kind: DOWNLOAD_ZIP_KIND.into(),
            status: JobStatus::Running,
            target: Value::Null,
            params,
            total,
            completed: 0,
            failed: 0,
            cancelled_at: None,
            created_at: String::new(),
            updated_at: String::new(),
            started_at: None,
            started_done: 0,
            eta_secs: None,
            item_ms: None,
        }
    }

    fn job_item(job: &JobRecord, position: i64, variant: i64) -> JobItemRecord {
        JobItemRecord {
            id: Uuid::new_v4(),
            job_id: job.id,
            asset_id: Uuid::new_v4().to_string(),
            status: JobItemStatus::Running,
            error: None,
            result: None,
            idempotency_key: None,
            attempts: 1,
            position,
            variant,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn a_plain_params_object_is_one_unnamed_variant() {
        let parsed =
            ExportVariants::parse(&json!({ "filename_template": "{seq}", "variants_x": 1 }))
                .unwrap();

        let [only] = parsed.0.as_slice() else {
            panic!("expected one variant, got {parsed:?}");
        };
        assert_eq!(only.preset, None);
        assert_eq!(
            only.params.params.filename_template.as_deref(),
            Some("{seq}")
        );
    }

    #[test]
    fn variants_keep_their_order_trimmed_names_and_own_params() {
        let parsed = ExportVariants::parse(&json!({ "variants": [
            variant("  Web 2048 ", json!({ "filename_template": "{name}_{preset}" })),
            variant("Print", json!({ "format": "tiff" })),
        ] }))
        .unwrap();

        let presets: Vec<Option<&str>> = parsed.0.iter().map(|v| v.preset.as_deref()).collect();
        assert_eq!(presets, [Some("Web 2048"), Some("Print")]);
        assert_eq!(
            parsed.0[0].params.params.filename_template.as_deref(),
            Some("{name}_{preset}")
        );
        assert_eq!(
            parsed.0[1].params.params.filename_template.as_deref(),
            Some("{name}_{preset}")
        );
        assert_eq!(parsed.count(), 2);
    }

    #[test]
    fn a_variant_without_a_template_names_files_by_preset() {
        let parsed = ExportVariants::parse(&json!({ "variants": [
            variant("Web", json!({ "filename_template": " " })),
            variant("Print", json!({ "filename_template": "{seq}" })),
        ] }))
        .unwrap();
        let single = ExportVariants::parse(&json!({})).unwrap();

        let templates: Vec<Option<&str>> = parsed
            .0
            .iter()
            .map(|v| v.params.params.filename_template.as_deref())
            .collect();
        assert_eq!(templates, [Some("{name}_{preset}"), Some("{seq}")]);
        assert_eq!(single.0[0].params.params.filename_template, None);
    }

    #[test]
    fn variants_take_no_other_top_level_key() {
        let params = json!({ "variants": [variant("Web", json!({}))], "format": "png" });

        assert_eq!(rejected(params), "unexpected key beside variants: format");
    }

    #[test]
    fn a_job_takes_one_to_sixteen_variants() {
        let many = |n: usize| -> Value {
            let list: Vec<Value> = (0..n)
                .map(|i| variant(&format!("P{i}"), json!({})))
                .collect();
            json!({ "variants": list })
        };

        for n in [0, 17] {
            assert_eq!(rejected(many(n)), "a job exports 1 to 16 presets", "{n}");
        }
        for n in [1, 16] {
            assert_eq!(ExportVariants::parse(&many(n)).unwrap().count(), n);
        }
    }

    #[test]
    fn preset_names_are_display_names_unique_ignoring_case() {
        for (presets, message) in [
            (vec!["  "], "preset name required"),
            (
                vec![&*"x".repeat(81)],
                "preset name longer than 80 characters",
            ),
            (vec!["Web", " web "], "preset web is listed twice"),
            (
                vec!["A/B", "a:b"],
                "presets A/B and a:b give the same file name",
            ),
        ] {
            let list: Vec<Value> = presets.iter().map(|p| variant(p, json!({}))).collect();

            assert_eq!(rejected(json!({ "variants": list })), message);
        }
    }

    #[test]
    fn replace_original_needs_a_single_variant() {
        let replace = json!({ "replace_original": true });
        let one = ExportVariants::parse(&json!({ "variants": [variant("Web", replace.clone())] }))
            .unwrap();
        let two = ExportVariants::parse(
            &json!({ "variants": [variant("Web", replace), variant("Print", json!({}))] }),
        )
        .unwrap();

        assert!(one.one_version_each().is_ok());
        assert!(matches!(
            two.one_version_each(),
            Err(AppError::BadRequest(message))
                if message == "replace original exports one version of each photo"
        ));
    }

    #[test]
    fn invalid_variant_params_name_their_preset() {
        let message = rejected(json!({ "variants": [variant("Web", json!({ "format": "bmp" }))] }));

        assert!(
            message.starts_with("Web: invalid export params"),
            "{message}"
        );
        assert!(
            rejected(json!({ "variants": [{ "preset": "Web" }] })).starts_with("invalid variants")
        );
        assert!(rejected(json!({ "variants": "Web" })).starts_with("invalid variants"));
    }

    #[test]
    fn items_run_each_photo_through_every_variant_in_order() {
        let items = expand_items(vec!["a".into(), "b".into()], 2);

        let order: Vec<(&str, i64, i64)> = items
            .iter()
            .map(|item| (item.asset_id.as_str(), item.position, item.variant))
            .collect();
        assert_eq!(order, [("a", 1, 0), ("a", 1, 1), ("b", 2, 0), ("b", 2, 1)]);
        assert_eq!(expand_items(vec!["a".into()], 1).len(), 1);
    }

    #[test]
    fn seq_counts_photos_not_items() {
        let params =
            json!({ "variants": [variant("Web", json!({})), variant("Print", json!({}))] });
        let job = job(params, 24);

        let export = item_export(&job, &job_item(&job, 3, 1)).unwrap();

        assert_eq!(
            export.seq,
            Seq {
                position: 3,
                total: 12
            }
        );
        assert_eq!(export.preset.as_deref(), Some("Print"));
        let name = NameTemplate::parse(Some("{seq}_{preset}"))
            .unwrap()
            .render(&NameContext {
                original: "DSC0001.ARW",
                date: None,
                seq: export.seq,
                preset: export.preset.as_deref(),
            });
        assert_eq!(name, "03_Print");
    }

    #[test]
    fn a_single_export_item_keeps_its_plain_naming_and_key() {
        let job = job(json!({ "format": "png" }), 12);
        let item = job_item(&job, 4, 0);

        let export = item_export(&job, &item).unwrap();

        assert_eq!(
            export.seq,
            Seq {
                position: 4,
                total: 12
            }
        );
        assert_eq!(export.preset, None);
        assert_eq!(export.params.params.filename_template, None);
        let name = NameTemplate::parse(Some("{seq}_{name}{preset}"))
            .unwrap()
            .render(&NameContext {
                original: "DSC0001.ARW",
                date: None,
                seq: export.seq,
                preset: export.preset.as_deref(),
            });
        assert_eq!(name, "04_DSC0001");
        assert_eq!(
            idempotency_key(job.id, item.variant),
            format!("job-{}", job.id)
        );
    }

    #[test]
    fn an_item_outside_the_variants_fails() {
        let job = job(json!({ "variants": [variant("Web", json!({}))] }), 1);

        for variant in [1, -1] {
            let error = item_export(&job, &job_item(&job, 1, variant))
                .err()
                .unwrap();
            assert_eq!(error.to_string(), format!("job has no variant {variant}"));
        }
    }

    #[test]
    fn later_variants_get_their_own_idempotency_key() {
        let id = Uuid::new_v4();

        assert_eq!(idempotency_key(id, 0), format!("job-{id}"));
        assert_eq!(idempotency_key(id, 1), format!("job-{id}-v1"));
        assert_eq!(idempotency_key(id, 15), format!("job-{id}-v15"));
    }

    #[test]
    fn results_name_their_preset_only_in_variant_jobs() {
        let result = json!({ "filename": "a.jpg" });

        assert_eq!(tag_preset(result.clone(), None), result);
        assert_eq!(
            tag_preset(result, Some("Web")),
            json!({ "filename": "a.jpg", "preset": "Web" })
        );
    }
}
