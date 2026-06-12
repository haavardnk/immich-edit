use std::convert::Infallible;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::job_store::{JobItemRecord, JobRecord, NewJobItem};
use crate::state::AppState;

const KNOWN_JOB_KINDS: &[&str] = &[];
const MAX_ITEMS: usize = 10_000;
const LIST_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub struct CreateJobBody {
    pub kind: String,
    #[serde(default)]
    pub target: serde_json::Value,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct JobDetail {
    pub job: JobRecord,
    pub items: Vec<JobItemRecord>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<JobRecord>>, AppError> {
    let jobs = state.jobs.list_jobs(LIST_LIMIT).await?;
    Ok(Json(jobs))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateJobBody>,
) -> Result<Json<JobRecord>, AppError> {
    let kind = body.kind.trim();
    if !KNOWN_JOB_KINDS.contains(&kind) {
        return Err(AppError::BadRequest(format!("unknown job kind: {kind}")));
    }
    if body.asset_ids.is_empty() {
        return Err(AppError::BadRequest("asset_ids required".into()));
    }
    if body.asset_ids.len() > MAX_ITEMS {
        return Err(AppError::BadRequest("too many items".into()));
    }
    let items: Vec<NewJobItem> = body
        .asset_ids
        .into_iter()
        .map(|asset_id| NewJobItem {
            asset_id,
            idempotency_key: None,
        })
        .collect();
    let job = state
        .jobs
        .create_job(kind, &body.target, &body.params, &items)
        .await?;
    Ok(Json(job))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobDetail>, AppError> {
    let job = state.jobs.get_job(id).await?.ok_or(AppError::NotFound)?;
    let items = state.jobs.list_items(id).await?;
    Ok(Json(JobDetail { job, items }))
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if state.jobs.cancel_job(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let job = state.jobs.get_job(id).await?.ok_or(AppError::NotFound)?;
    let rx = state.jobs.subscribe();
    let snapshot = tokio_stream::once(Ok(job_event(&job)));
    let updates = BroadcastStream::new(rx).filter_map(move |res| match res {
        Ok(rec) if rec.id == id => Some(Ok(job_event(&rec))),
        _ => None,
    });
    Ok(Sse::new(snapshot.chain(updates)).keep_alive(KeepAlive::default()))
}

fn job_event(job: &JobRecord) -> Event {
    let data = serde_json::to_string(job).unwrap_or_else(|_| "{}".into());
    Event::default().event("job").data(data)
}
