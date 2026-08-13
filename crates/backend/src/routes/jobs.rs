use std::convert::Infallible;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::apply_preset::APPLY_PRESET_KIND;
use crate::services::copy_expand::expand_search;
use crate::services::export::{
    DOWNLOAD_ZIP_KIND, EXPORT_JOB_KIND, build_zip_archive, cleanup_zip_job,
};
use crate::services::job_store::{JobItemRecord, JobRecord, JobStatus, NewJob, NewJobItem};
use crate::services::paste_edits::PASTE_EDITS_KIND;
use crate::services::reset_edits::RESET_EDITS_KIND;
use crate::state::AppState;

const KNOWN_JOB_KINDS: &[&str] = &[
    EXPORT_JOB_KIND,
    DOWNLOAD_ZIP_KIND,
    APPLY_PRESET_KIND,
    PASTE_EDITS_KIND,
    RESET_EDITS_KIND,
];
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

pub async fn list(
    State(state): State<AppState>,
    ctx: AuthCtx,
) -> Result<Json<Vec<JobRecord>>, AppError> {
    let jobs = state.jobs.list_jobs(ctx.owner, LIST_LIMIT).await?;
    Ok(Json(jobs))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<CreateJobBody>,
) -> Result<Json<JobRecord>, AppError> {
    let kind = body.kind.trim();
    if !KNOWN_JOB_KINDS.contains(&kind) {
        return Err(AppError::BadRequest(format!("unknown job kind: {kind}")));
    }
    let asset_ids = if body.asset_ids.is_empty() {
        match body.target.get("search") {
            Some(query) => {
                expand_search(&state.edits, &ctx.immich, ctx.owner, query, MAX_ITEMS).await?
            }
            None => {
                return Err(AppError::BadRequest(
                    "asset_ids or target.search required".into(),
                ));
            }
        }
    } else {
        body.asset_ids
    };
    if asset_ids.is_empty() {
        return Err(AppError::BadRequest("no matching assets".into()));
    }
    if asset_ids.len() > MAX_ITEMS {
        return Err(AppError::BadRequest("too many items".into()));
    }
    let items: Vec<NewJobItem> = asset_ids
        .into_iter()
        .map(|asset_id| NewJobItem {
            asset_id,
            idempotency_key: None,
        })
        .collect();
    let job = state
        .jobs
        .create_job(NewJob {
            owner: ctx.owner,
            server_epoch: ctx.server_epoch,
            auth_session_id: ctx.session_id,
            kind,
            target: &body.target,
            params: &body.params,
            items: &items,
            cred: ctx.cred.as_slice(),
            auth_kind: ctx.auth_kind,
        })
        .await?;
    Ok(Json(job))
}

async fn owned_job(state: &AppState, ctx: &AuthCtx, id: Uuid) -> Result<JobRecord, AppError> {
    let job = state.jobs.get_job(id).await?.ok_or(AppError::NotFound)?;
    if job.user_id != ctx.owner {
        return Err(AppError::NotFound);
    }
    Ok(job)
}

pub async fn get(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Json<JobDetail>, AppError> {
    let job = owned_job(&state, &ctx, id).await?;
    let items = state.jobs.list_items(id).await?;
    Ok(Json(JobDetail { job, items }))
}

pub async fn cancel(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    owned_job(&state, &ctx, id).await?;
    if state.jobs.cancel_job(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn clear(State(state): State<AppState>, ctx: AuthCtx) -> Result<StatusCode, AppError> {
    let cleared = state.jobs.clear_finished(ctx.owner).await?;
    for (id, kind) in cleared {
        if kind == DOWNLOAD_ZIP_KIND {
            cleanup_zip_job(&state, ctx.server_epoch, ctx.owner, id).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn download(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let job = owned_job(&state, &ctx, id).await?;
    if job.kind != DOWNLOAD_ZIP_KIND {
        return Err(AppError::NotFound);
    }
    if job.status != JobStatus::Completed {
        return Err(AppError::BadRequest("job not complete".into()));
    }
    let archive = build_zip_archive(&state, ctx.server_epoch, ctx.owner, id).await?;
    let file = tokio::fs::File::open(&archive).await.map_err(|e| {
        tracing::error!(error = %e, "open zip archive");
        AppError::Internal
    })?;
    let body = Body::from_stream(ReaderStream::new(file));
    let short: String = id.to_string().chars().take(8).collect();
    let mut resp = Response::new(body);
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"immich-edit-{short}.zip\""))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );
    Ok(resp)
}

pub async fn events(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let job = owned_job(&state, &ctx, id).await?;
    let rx = state.jobs.subscribe();
    let snapshot = tokio_stream::once(Ok(job_event(&job)));
    let owner = ctx.owner;
    let updates = BroadcastStream::new(rx).filter_map(move |res| match res {
        Ok(rec) if rec.id == id && rec.user_id == owner => Some(Ok(job_event(&rec))),
        _ => None,
    });
    Ok(Sse::new(snapshot.chain(updates)).keep_alive(KeepAlive::default()))
}

fn job_event(job: &JobRecord) -> Event {
    let data = serde_json::to_string(job).unwrap_or_else(|_| "{}".into());
    Event::default().event("job").data(data)
}
