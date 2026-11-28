use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::dto::{AlbumPatch, AlbumSummary, BulkIdResponse};
use crate::names::{album_description, display_name};
use crate::routes::auth::AuthCtx;
use crate::state::AppState;

const MAX_ALBUM_ASSETS: usize = 10_000;

#[derive(Deserialize)]
pub struct AlbumAssetsBody {
    ids: Vec<AssetKey>,
}

#[derive(Deserialize)]
pub struct AlbumListQuery {
    #[serde(rename = "assetId")]
    asset_id: Option<AssetKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAlbumBody {
    #[serde(rename = "albumName")]
    album_name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateAlbumBody {
    #[serde(rename = "albumName", default)]
    album_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "albumThumbnailAssetId", default)]
    album_thumbnail_asset_id: Option<AssetKey>,
}

fn description_of(raw: Option<String>) -> Result<Option<Option<String>>, AppError> {
    raw.map(|description| album_description(&description))
        .transpose()
        .map_err(AppError::BadRequest)
}

impl TryFrom<UpdateAlbumBody> for AlbumPatch {
    type Error = AppError;

    fn try_from(body: UpdateAlbumBody) -> Result<Self, AppError> {
        let album_name = body
            .album_name
            .map(|name| display_name(&name, "album name"))
            .transpose()
            .map_err(AppError::BadRequest)?;
        let description = description_of(body.description)?;
        let patch = Self {
            album_name,
            description: description.map(Option::unwrap_or_default),
            album_thumbnail_asset_id: body.album_thumbnail_asset_id.as_ref().map(AssetKey::source),
        };
        if patch.is_empty() {
            return Err(AppError::BadRequest("nothing to update".into()));
        }
        Ok(patch)
    }
}

pub async fn list(
    ctx: AuthCtx,
    Query(query): Query<AlbumListQuery>,
) -> Result<Json<Vec<AlbumSummary>>, AppError> {
    let albums = match query.asset_id {
        Some(asset) => ctx.immich.albums_with_asset(asset.source()).await?,
        None => ctx.immich.list_albums().await?,
    };
    Ok(Json(albums))
}

pub async fn create(
    ctx: AuthCtx,
    Json(body): Json<CreateAlbumBody>,
) -> Result<Json<AlbumSummary>, AppError> {
    let name = display_name(&body.album_name, "album name").map_err(AppError::BadRequest)?;
    let description = description_of(body.description)?.flatten();
    let album = ctx
        .immich
        .create_album(&name, description.as_deref())
        .await?;
    Ok(Json(album))
}

pub async fn detail(ctx: AuthCtx, Path(id): Path<Uuid>) -> Result<Json<AlbumSummary>, AppError> {
    let album = ctx.immich.album(id).await?;
    Ok(Json(album))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAlbumBody>,
) -> Result<Json<AlbumSummary>, AppError> {
    let patch = AlbumPatch::try_from(body)?;
    ctx.immich.update_album(id, &patch).await?;
    state.search_caches.names_changed();
    Ok(Json(ctx.immich.album(id).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ctx.immich.delete_album(id).await?;
    state.search_caches.membership_changed();
    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ctx.immich.leave_album(id, ctx.owner).await?;
    state.search_caches.membership_changed();
    Ok(StatusCode::NO_CONTENT)
}

fn source_ids(body: AlbumAssetsBody) -> Result<Vec<Uuid>, AppError> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("no assets".into()));
    }
    if body.ids.len() > MAX_ALBUM_ASSETS {
        return Err(AppError::BadRequest("too many assets".into()));
    }
    let mut ids: Vec<Uuid> = body.ids.iter().map(AssetKey::source).collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

pub async fn add_assets(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<AlbumAssetsBody>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let ids = source_ids(body)?;
    let resp = ctx.immich.add_assets_to_album(id, &ids).await?;
    state.search_caches.membership_changed();
    Ok(Json(resp))
}

pub async fn remove_assets(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<AlbumAssetsBody>,
) -> Result<Json<Vec<BulkIdResponse>>, AppError> {
    let ids = source_ids(body)?;
    let resp = ctx.immich.remove_assets_from_album(id, &ids).await?;
    state.search_caches.membership_changed();
    Ok(Json(resp))
}
