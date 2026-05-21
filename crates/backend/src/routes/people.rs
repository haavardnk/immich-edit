use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::PersonSummary;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<PersonSummary>>, AppError> {
    let people = state.immich.list_people(true).await?;
    Ok(Json(people))
}

pub async fn thumbnail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let (bytes, ct) = state.immich.person_thumb(id).await?;
    let resp = Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_str(&ct).unwrap())
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(bytes))
        .unwrap();
    Ok(resp)
}
