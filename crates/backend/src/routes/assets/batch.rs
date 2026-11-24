use axum::Json;
use axum::http::StatusCode;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::dto::AssetSummary;
use crate::routes::auth::AuthCtx;
use crate::routes::map::checked_point;

const MAX_LOCATION_ASSETS: usize = 1000;
const MAX_SUMMARIES: usize = 100;
const SUMMARY_CONCURRENCY: usize = 8;
const LOCATION_CONCURRENCY: usize = 4;

fn sources(ids: &[AssetKey], max: usize) -> Result<Vec<Uuid>, AppError> {
    if ids.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if ids.len() > max {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut sources: Vec<Uuid> = Vec::with_capacity(ids.len());
    for id in ids {
        if !sources.contains(&id.source()) {
            sources.push(id.source());
        }
    }
    Ok(sources)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationBody {
    ids: Vec<AssetKey>,
    latitude: f64,
    longitude: f64,
}

pub async fn location(
    ctx: AuthCtx,
    Json(body): Json<LocationBody>,
) -> Result<StatusCode, AppError> {
    let (latitude, longitude) = checked_point(body.latitude, body.longitude)?;
    let ids = sources(&body.ids, MAX_LOCATION_ASSETS)?;
    let update = serde_json::json!({ "ids": ids, "latitude": latitude, "longitude": longitude });
    ctx.immich.update_assets(&update).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocatedAsset {
    id: AssetKey,
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationsBody {
    items: Vec<LocatedAsset>,
}

#[derive(Serialize)]
pub struct LocationsResult {
    failed: Vec<Uuid>,
}

struct PointGroup {
    latitude: f64,
    longitude: f64,
    ids: Vec<Uuid>,
}

fn point_groups(items: &[LocatedAsset]) -> Result<Vec<PointGroup>, AppError> {
    if items.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if items.len() > MAX_LOCATION_ASSETS {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut seen: Vec<Uuid> = Vec::with_capacity(items.len());
    let mut groups: Vec<PointGroup> = Vec::new();
    for item in items {
        let (latitude, longitude) = checked_point(item.latitude, item.longitude)?;
        let source = item.id.source();
        if seen.contains(&source) {
            continue;
        }
        seen.push(source);
        let same = |group: &&mut PointGroup| {
            group.latitude.to_bits() == latitude.to_bits()
                && group.longitude.to_bits() == longitude.to_bits()
        };
        match groups.iter_mut().find(same) {
            Some(group) => group.ids.push(source),
            None => groups.push(PointGroup {
                latitude,
                longitude,
                ids: vec![source],
            }),
        }
    }
    Ok(groups)
}

pub async fn locations(
    ctx: AuthCtx,
    Json(body): Json<LocationsBody>,
) -> Result<Json<LocationsResult>, AppError> {
    let groups = point_groups(&body.items)?;
    let mut failed: Vec<Uuid> = Vec::new();
    let mut first_error: Option<ImmichError> = None;
    let mut written = 0usize;
    for chunk in groups.chunks(LOCATION_CONCURRENCY) {
        let results = join_all(chunk.iter().map(|group| {
            let update = serde_json::json!({
                "ids": group.ids,
                "latitude": group.latitude,
                "longitude": group.longitude
            });
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
        _ => Ok(Json(LocationsResult { failed })),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummariesBody {
    ids: Vec<AssetKey>,
}

pub async fn summaries(
    ctx: AuthCtx,
    Json(body): Json<SummariesBody>,
) -> Result<Json<Vec<AssetSummary>>, AppError> {
    let ids = sources(&body.ids, MAX_SUMMARIES)?;
    let mut found: Vec<AssetSummary> = Vec::with_capacity(ids.len());
    for chunk in ids.chunks(SUMMARY_CONCURRENCY) {
        let results = join_all(chunk.iter().map(|id| ctx.immich.asset_summary(*id))).await;
        for result in results {
            match result {
                Ok(asset) if asset.is_image() => found.push(asset),
                Ok(_) | Err(ImmichError::NotFound) => {}
                Err(err) => return Err(err.into()),
            }
        }
    }
    Ok(Json(found))
}
