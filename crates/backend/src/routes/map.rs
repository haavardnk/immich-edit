use std::collections::HashSet;

use axum::Json;
use axum::extract::{Query, State};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::{MapMarker, PlaceName};
use crate::immich::{ImmichClient, ImmichResult};
use crate::routes::auth::AuthCtx;
use crate::services::map::{Place, StyleSource, resolve_styles};
use crate::state::AppState;

const VIDEO_PAGE_SIZE: u32 = 1000;
const MAX_VIDEO_PAGES: u32 = 50;

mod admin;

pub use admin::{settings, update_settings};

#[derive(Serialize)]
pub struct MapConfig {
    light: Option<String>,
    dark: Option<String>,
    source: StyleSource,
    search: bool,
}

pub async fn config(State(state): State<AppState>, ctx: AuthCtx) -> Json<MapConfig> {
    let caps = state.capabilities.get(ctx.server_epoch, &ctx.immich).await;
    let settings = state.map.settings();
    let styles = resolve_styles(&settings, &caps);
    Json(MapConfig {
        light: styles.light,
        dark: styles.dark,
        source: styles.source,
        search: !settings.geocoder_url.is_empty(),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerQuery {
    #[serde(default)]
    favorite: bool,
    after: Option<String>,
    before: Option<String>,
}

fn instant(raw: Option<String>, label: &str) -> Result<Option<String>, AppError> {
    raw.map(|value| {
        DateTime::parse_from_rfc3339(&value)
            .map(|at| at.to_rfc3339())
            .map_err(|_| AppError::BadRequest(format!("{label} must be an RFC 3339 time")))
    })
    .transpose()
}

async fn video_ids(immich: &ImmichClient, favorite: bool) -> ImmichResult<HashSet<Uuid>> {
    let mut ids = HashSet::new();
    let mut page: u32 = 1;
    for _ in 0..MAX_VIDEO_PAGES {
        let mut body = json!({
            "type": "VIDEO",
            "size": VIDEO_PAGE_SIZE,
            "page": page,
            "withExif": false,
        });
        if favorite {
            body["isFavorite"] = json!(true);
        }
        let res = immich.search_metadata(&body).await?;
        if res.items.is_empty() {
            break;
        }
        ids.extend(res.items.into_iter().map(|a| a.id.source()));
        let Some(next) = res
            .next_page
            .as_deref()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&n| n > page)
        else {
            break;
        };
        page = next;
    }
    Ok(ids)
}

pub async fn markers(
    ctx: AuthCtx,
    Query(query): Query<MarkerQuery>,
) -> Result<Json<Vec<MapMarker>>, AppError> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if query.favorite {
        params.push(("isFavorite", "true".into()));
    }
    if let Some(after) = instant(query.after, "after")? {
        params.push(("fileCreatedAfter", after));
    }
    if let Some(before) = instant(query.before, "before")? {
        params.push(("fileCreatedBefore", before));
    }
    let (mut markers, videos) = tokio::try_join!(
        ctx.immich.map_markers(&params),
        video_ids(&ctx.immich, query.favorite),
    )?;
    if !videos.is_empty() {
        markers.retain(|m| !videos.contains(&m.id));
    }
    Ok(Json(markers))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointQuery {
    lat: f64,
    lon: f64,
}

pub fn checked_point(lat: f64, lon: f64) -> Result<(f64, f64), AppError> {
    let valid = lat.is_finite() && lon.is_finite() && lat.abs() <= 90.0 && lon.abs() <= 180.0;
    if !valid {
        return Err(AppError::BadRequest(
            "latitude must be -90 to 90 and longitude -180 to 180".into(),
        ));
    }
    Ok((lat, lon))
}

pub async fn reverse_geocode(
    ctx: AuthCtx,
    Query(point): Query<PointQuery>,
) -> Result<Json<Option<PlaceName>>, AppError> {
    let (lat, lon) = checked_point(point.lat, point.lon)?;
    Ok(Json(ctx.immich.reverse_geocode(lat, lon).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    q: String,
}

pub async fn search(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Place>>, AppError> {
    let base = state.map.settings().geocoder_url;
    Ok(Json(
        state
            .map
            .geocoder
            .search(&base, ctx.owner, &query.q)
            .await?,
    ))
}
