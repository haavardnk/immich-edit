use std::collections::HashSet;

use axum::Json;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::routes::auth::AuthCtx;

const MAX_ASSETS: usize = 1000;
const MAX_SHIFT_MINUTES: i64 = 48 * 60;
const MAX_ZONE_LEN: usize = 64;
const CONCURRENCY: usize = 4;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TimeShift {
    id: AssetKey,
    minutes: i64,
    #[serde(default)]
    time_zone: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureTimeBody {
    items: Vec<TimeShift>,
}

#[derive(Serialize)]
pub struct CaptureTimeResult {
    failed: Vec<Uuid>,
}

#[derive(PartialEq)]
struct ShiftKey {
    minutes: i64,
    time_zone: Option<String>,
}

struct ShiftGroup {
    key: ShiftKey,
    ids: Vec<Uuid>,
}

fn checked_zone(zone: Option<String>) -> Result<Option<String>, AppError> {
    let Some(zone) = zone else { return Ok(None) };
    let valid = !zone.is_empty()
        && zone.len() <= MAX_ZONE_LEN
        && zone
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'));
    if !valid {
        return Err(AppError::BadRequest(format!("bad time zone {zone}")));
    }
    Ok(Some(zone))
}

fn shift_groups(items: Vec<TimeShift>) -> Result<Vec<ShiftGroup>, AppError> {
    if items.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if items.len() > MAX_ASSETS {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut groups: Vec<ShiftGroup> = Vec::new();
    for item in items {
        if item.minutes.abs() > MAX_SHIFT_MINUTES {
            return Err(AppError::BadRequest(format!(
                "a shift must be within {MAX_SHIFT_MINUTES} minutes"
            )));
        }
        let key = ShiftKey {
            minutes: item.minutes,
            time_zone: checked_zone(item.time_zone)?,
        };
        let source = item.id.source();
        if key.minutes == 0 && key.time_zone.is_none() {
            continue;
        }
        if !seen.insert(source) {
            continue;
        }
        match groups.iter_mut().find(|group| group.key == key) {
            Some(group) => group.ids.push(source),
            None => groups.push(ShiftGroup {
                key,
                ids: vec![source],
            }),
        }
    }
    Ok(groups)
}

pub async fn capture_time(
    ctx: AuthCtx,
    Json(body): Json<CaptureTimeBody>,
) -> Result<Json<CaptureTimeResult>, AppError> {
    let groups = shift_groups(body.items)?;
    let mut failed: Vec<Uuid> = Vec::new();
    let mut first_error: Option<ImmichError> = None;
    let mut written = 0usize;
    for chunk in groups.chunks(CONCURRENCY) {
        let results = join_all(chunk.iter().map(|group| {
            let mut update = serde_json::json!({
                "ids": group.ids,
                "dateTimeRelative": group.key.minutes
            });
            if let Some(zone) = &group.key.time_zone {
                update["timeZone"] = serde_json::Value::String(zone.clone());
            }
            let immich = ctx.immich.clone();
            async move { immich.update_assets(&update).await }
        }))
        .await;
        for (group, result) in chunk.iter().zip(results) {
            match result {
                Ok(()) => written += group.ids.len(),
                Err(err) => {
                    failed.extend(&group.ids);
                    first_error.get_or_insert(err);
                }
            }
        }
    }
    match first_error {
        Some(err) if written == 0 => Err(err.into()),
        _ => Ok(Json(CaptureTimeResult { failed })),
    }
}
