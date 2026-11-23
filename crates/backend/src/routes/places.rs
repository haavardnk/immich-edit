use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::PlaceName;
use crate::names::display_name;
use crate::routes::auth::AuthCtx;
use crate::routes::map::checked_point;
use crate::services::edits_store::{MAX_SAVED_PLACES, PlaceWrite, Point, RecentPlace, SavedPlace};
use crate::state::AppState;

const MAX_PLACE_PART_CHARS: usize = 200;

#[derive(Serialize)]
pub struct Places {
    saved: Vec<SavedPlace>,
    recent: Vec<RecentPlace>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpotBody {
    lat: f64,
    lon: f64,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedPlaceBody {
    name: String,
    lat: f64,
    lon: f64,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    country: Option<String>,
}

impl SavedPlaceBody {
    fn split(self) -> (String, SpotBody) {
        let spot = SpotBody {
            lat: self.lat,
            lon: self.lon,
            city: self.city,
            state: self.state,
            country: self.country,
        };
        (self.name, spot)
    }
}

fn place_part(raw: Option<String>, label: &str) -> Result<Option<String>, AppError> {
    let Some(text) = raw.map(|value| value.trim().to_string()) else {
        return Ok(None);
    };
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > MAX_PLACE_PART_CHARS || text.chars().any(char::is_control) {
        return Err(AppError::BadRequest(format!(
            "{label} must be at most {MAX_PLACE_PART_CHARS} characters without control characters"
        )));
    }
    Ok(Some(text))
}

fn parse_spot(spot: SpotBody) -> Result<(Point, PlaceName), AppError> {
    let (lat, lon) = checked_point(spot.lat, spot.lon)?;
    let place = PlaceName {
        city: place_part(spot.city, "city")?,
        state: place_part(spot.state, "state")?,
        country: place_part(spot.country, "country")?,
    };
    Ok((Point { lat, lon }, place))
}

fn saved(write: PlaceWrite, name: &str) -> Result<Json<SavedPlace>, AppError> {
    match write {
        PlaceWrite::Saved(record) => Ok(Json(record)),
        PlaceWrite::Duplicate => Err(AppError::Conflict(format!(
            "You already have a place called {name}"
        ))),
        PlaceWrite::Full => Err(AppError::Conflict(format!(
            "You have {MAX_SAVED_PLACES} saved places. Remove one first."
        ))),
        PlaceWrite::Missing => Err(AppError::NotFound),
    }
}

pub async fn list(State(state): State<AppState>, ctx: AuthCtx) -> Result<Json<Places>, AppError> {
    Ok(Json(Places {
        saved: state.edits.list_saved_places(ctx.owner).await?,
        recent: state.edits.list_recent_places(ctx.owner).await?,
    }))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<SavedPlaceBody>,
) -> Result<Json<SavedPlace>, AppError> {
    let (raw, spot) = body.split();
    let name = display_name(&raw, "place name").map_err(AppError::BadRequest)?;
    let (at, place) = parse_spot(spot)?;
    let write = state
        .edits
        .create_saved_place(ctx.owner, &name, at, &place)
        .await?;
    saved(write, &name)
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<SavedPlaceBody>,
) -> Result<Json<SavedPlace>, AppError> {
    let (raw, spot) = body.split();
    let name = display_name(&raw, "place name").map_err(AppError::BadRequest)?;
    let (at, place) = parse_spot(spot)?;
    let write = state
        .edits
        .update_saved_place(ctx.owner, id, &name, at, &place)
        .await?;
    saved(write, &name)
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.edits.delete_saved_place(ctx.owner, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn used(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.edits.touch_saved_place(ctx.owner, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn remember(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<SpotBody>,
) -> Result<Json<Vec<RecentPlace>>, AppError> {
    let (at, place) = parse_spot(body)?;
    Ok(Json(
        state.edits.remember_place(ctx.owner, at, &place).await?,
    ))
}

pub async fn clear_recent(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<StatusCode, AppError> {
    state.edits.clear_recent_places(ctx.owner).await?;
    Ok(StatusCode::NO_CONTENT)
}
