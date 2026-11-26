use std::collections::HashSet;

use axum::Json;
use chrono::{DateTime, Datelike, FixedOffset, SecondsFormat, SubsecRound, Utc};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::routes::auth::AuthCtx;

const MAX_ASSETS: usize = 1000;
const MAX_SHIFT_MINUTES: i64 = 200 * 366 * 24 * 60;
const SHIFTS: std::ops::RangeInclusive<i64> = -MAX_SHIFT_MINUTES..=MAX_SHIFT_MINUTES;
const UTC_YEARS: std::ops::RangeInclusive<i32> = 1801..=9998;
const MAX_ZONE_LEN: usize = 64;
const CONCURRENCY: usize = 4;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TimeChange {
    id: AssetKey,
    #[serde(default)]
    minutes: Option<i64>,
    #[serde(default)]
    at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    time_zone: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureTimeBody {
    items: Vec<TimeChange>,
}

#[derive(Serialize)]
pub struct CaptureTimeResult {
    failed: Vec<Uuid>,
}

#[derive(PartialEq)]
enum Change {
    Shift(i64),
    Set(DateTime<FixedOffset>),
}

#[derive(PartialEq)]
struct ChangeKey {
    change: Change,
    time_zone: Option<String>,
}

struct ChangeGroup {
    key: ChangeKey,
    ids: Vec<Uuid>,
}

impl ChangeGroup {
    fn update(&self) -> Value {
        let mut update = json!({ "ids": self.ids });
        match self.key.change {
            Change::Shift(minutes) => update["dateTimeRelative"] = json!(minutes),
            Change::Set(at) => {
                update["dateTimeOriginal"] = json!(at.to_rfc3339_opts(SecondsFormat::Millis, false))
            }
        }
        if let Some(zone) = &self.key.time_zone {
            update["timeZone"] = json!(zone);
        }
        update
    }
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

fn checked_key(item: TimeChange) -> Result<ChangeKey, AppError> {
    let time_zone = checked_zone(item.time_zone)?;
    let change = match (item.minutes, item.at) {
        (Some(minutes), None) if SHIFTS.contains(&minutes) => Change::Shift(minutes),
        (Some(_), None) => {
            return Err(AppError::BadRequest(format!(
                "a shift must be within {MAX_SHIFT_MINUTES} minutes"
            )));
        }
        (None, Some(at)) if !UTC_YEARS.contains(&at.with_timezone(&Utc).year()) => {
            return Err(AppError::BadRequest(format!("{at} is out of range")));
        }
        (None, Some(_)) if time_zone.is_none() => {
            return Err(AppError::BadRequest(
                "a set time needs its time zone".into(),
            ));
        }
        (None, Some(at)) => Change::Set(at.trunc_subsecs(3)),
        _ => {
            return Err(AppError::BadRequest(
                "give either minutes or at for each photo".into(),
            ));
        }
    };
    Ok(ChangeKey { change, time_zone })
}

fn change_groups(items: Vec<TimeChange>) -> Result<Vec<ChangeGroup>, AppError> {
    if items.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if items.len() > MAX_ASSETS {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut groups: Vec<ChangeGroup> = Vec::new();
    for item in items {
        let source = item.id.source();
        let key = checked_key(item)?;
        if key.change == Change::Shift(0) && key.time_zone.is_none() {
            continue;
        }
        if !seen.insert(source) {
            continue;
        }
        match groups.iter_mut().find(|group| group.key == key) {
            Some(group) => group.ids.push(source),
            None => groups.push(ChangeGroup {
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
    let groups = change_groups(body.items)?;
    let mut failed: Vec<Uuid> = Vec::new();
    let mut first_error: Option<ImmichError> = None;
    let mut written = 0usize;
    for chunk in groups.chunks(CONCURRENCY) {
        let results = join_all(chunk.iter().map(|group| {
            let update = group.update();
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
