use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::membership::{Albums, MembershipCache, Reader, Source, Tags};
use crate::state::AppState;

const MAX_ASSETS: usize = 100;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipBody {
    ids: Vec<AssetKey>,
}

fn sources(body: MembershipBody) -> Result<Vec<Uuid>, AppError> {
    let mut ids: Vec<Uuid> = body.ids.iter().map(AssetKey::source).collect();
    ids.sort_unstable();
    ids.dedup();
    match ids.len() {
        0 => Err(AppError::BadRequest("no assets".into())),
        n if n > MAX_ASSETS => Err(AppError::BadRequest(format!("at most {MAX_ASSETS} assets"))),
        _ => Ok(ids),
    }
}

async fn answer<S: Source>(
    cache: &MembershipCache<S>,
    ctx: &AuthCtx,
    body: MembershipBody,
) -> Result<Json<BTreeMap<Uuid, S::Value>>, AppError> {
    let assets = sources(body)?;
    let reader = Reader {
        immich: &ctx.immich,
        server_epoch: ctx.server_epoch,
        owner: ctx.owner,
    };
    Ok(Json(cache.of(reader, assets).await?))
}

pub async fn albums(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<MembershipBody>,
) -> Result<Json<BTreeMap<Uuid, <Albums as Source>::Value>>, AppError> {
    answer(state.search_caches.album_membership(), &ctx, body).await
}

pub async fn tags(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<MembershipBody>,
) -> Result<Json<BTreeMap<Uuid, <Tags as Source>::Value>>, AppError> {
    answer(state.search_caches.tag_membership(), &ctx, body).await
}
