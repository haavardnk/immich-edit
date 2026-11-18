use axum::body::Body;
use axum::http::Request;
use immich_edit_backend::services::auth_store::AuthKind;
use immich_edit_backend::services::job_store::{JobItemRecord, JobRecord, NewJob, NewJobItem};
use immich_edit_backend::state::AppState;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{TEST_API_KEY, asset_id, empty_request, get};

pub fn download_request(id: Uuid) -> Request<Body> {
    get(&format!("/api/jobs/{id}/download"))
}

pub fn retry_request(id: Uuid) -> Request<Body> {
    empty_request("POST", &format!("/api/jobs/{id}/retry"))
}

pub async fn complete_next(state: &AppState) -> JobItemRecord {
    let item = state.jobs.claim_next_item().await.unwrap().unwrap();
    state
        .jobs
        .complete_item(item.id, &json!({ "ok": true }))
        .await
        .unwrap();
    item
}

pub async fn fail_next(state: &AppState) -> JobItemRecord {
    let item = state.jobs.claim_next_item().await.unwrap().unwrap();
    state.jobs.fail_item(item.id, "boom").await.unwrap();
    item
}

pub struct JobSeed {
    kind: String,
    params: Value,
    assets: Vec<String>,
    server_epoch: Option<i64>,
}

impl JobSeed {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            params: json!({}),
            assets: vec![asset_id().to_string()],
            server_epoch: None,
        }
    }

    pub fn params(mut self, params: Value) -> Self {
        self.params = params;
        self
    }

    pub fn assets<T: ToString>(mut self, assets: &[T]) -> Self {
        self.assets = assets.iter().map(ToString::to_string).collect();
        self
    }

    pub fn server_epoch(mut self, server_epoch: i64) -> Self {
        self.server_epoch = Some(server_epoch);
        self
    }

    pub async fn create(self, state: &AppState, token: &str) -> JobRecord {
        let ctx = state.auth.authenticate(token).await.unwrap().unwrap();
        let items: Vec<NewJobItem> = (1_i64..)
            .zip(self.assets)
            .map(|(position, asset_id)| NewJobItem {
                asset_id,
                idempotency_key: None,
                position,
                variant: 0,
            })
            .collect();
        state
            .jobs
            .create_job(NewJob {
                owner: ctx.user.id,
                server_epoch: self.server_epoch.unwrap_or(ctx.server_epoch),
                auth_session_id: ctx.session_id,
                kind: &self.kind,
                target: &json!({}),
                params: &self.params,
                items: &items,
                cred: TEST_API_KEY.as_bytes(),
                auth_kind: AuthKind::ApiKey,
            })
            .await
            .unwrap()
    }
}
