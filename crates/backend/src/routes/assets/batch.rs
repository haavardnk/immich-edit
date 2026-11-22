use axum::Json;
use futures_util::future::join_all;
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::dto::AssetSummary;
use crate::routes::auth::AuthCtx;

const MAX_SUMMARIES: usize = 100;
const SUMMARY_CONCURRENCY: usize = 8;

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
                Ok(asset) => found.push(asset),
                Err(ImmichError::NotFound) => {}
                Err(err) => return Err(err.into()),
            }
        }
    }
    Ok(Json(found))
}
