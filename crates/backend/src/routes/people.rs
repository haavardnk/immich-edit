use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::dto::PersonSummary;
use crate::routes::auth::AuthCtx;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<PersonSummary>>, AppError> {
    let mut people = ctx.immich.list_people(true).await?;
    let ids: Vec<Uuid> = people.iter().map(|p| p.id).collect();
    let counts = state
        .people_counts
        .counts_for(ctx.owner, ctx.server_epoch, &ctx.immich, &ids)
        .await;
    for person in &mut people {
        person.asset_count = counts.get(&person.id).copied();
    }
    Ok(Json(people))
}

pub async fn thumbnail(ctx: AuthCtx, Path(id): Path<Uuid>) -> Result<Response, AppError> {
    let (bytes, ct) = ctx.immich.person_thumb(id).await?;
    let resp = Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_str(&ct).unwrap())
        .header(header::CACHE_CONTROL, "private, max-age=86400")
        .body(Body::from(bytes))
        .unwrap();
    Ok(resp)
}
