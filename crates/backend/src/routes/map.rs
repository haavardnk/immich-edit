use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::immich::dto::{MapMarker, PlaceName};
use crate::routes::auth::AuthCtx;
use crate::routes::search::searcher;
use crate::services::map::{Place, StyleSource, resolve_styles};
use crate::services::search::filter::BrowseFilter;
use crate::state::AppState;

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

fn instant(raw: Option<String>, label: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    raw.map(|value| {
        DateTime::parse_from_rfc3339(&value)
            .map(|at| at.with_timezone(&Utc))
            .map_err(|_| AppError::BadRequest(format!("{label} must be an RFC 3339 time")))
    })
    .transpose()
}

pub async fn markers(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Query(query): Query<MarkerQuery>,
) -> Result<Json<Vec<MapMarker>>, AppError> {
    let filter = BrowseFilter {
        favorite: query.favorite,
        taken_after: instant(query.after, "after")?,
        taken_before: instant(query.before, "before")?,
        ..BrowseFilter::default()
    };
    let mut params: Vec<(&str, String)> = Vec::new();
    if filter.favorite {
        params.push(("isFavorite", "true".into()));
    }
    if let Some(after) = filter.taken_after {
        params.push(("fileCreatedAfter", after.to_rfc3339()));
    }
    if let Some(before) = filter.taken_before {
        params.push(("fileCreatedBefore", before.to_rfc3339()));
    }
    let searcher = searcher(&state, &ctx).await;
    let (markers, videos) = tokio::try_join!(
        async { Ok::<_, AppError>(ctx.immich.map_markers(&params).await?) },
        searcher.video_ids(filter),
    )?;
    Ok(Json(
        markers
            .into_iter()
            .filter(|marker| !videos.contains(&marker.id))
            .collect(),
    ))
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
