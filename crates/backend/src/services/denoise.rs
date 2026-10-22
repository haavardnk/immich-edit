mod error;
mod jobs;
mod worker;

use std::sync::Arc;
use std::time::Duration;

use raw_pipeline::ai_denoise::DenoisePlane;
use serde::{Serialize, Serializer};
use tokio::sync::{Mutex, Semaphore, watch};
use uuid::Uuid;

use crate::config::{Config, MlRuntimeMode};
use crate::immich::ImmichClient;
use crate::services::denoise_store::PlaneKey;
use crate::services::ml_runtime::InferencePermits;
use crate::services::model_store::ModelStore;
use crate::services::render::{RenderIdentity, RenderService};

pub use error::{BUSY_RETRY_SECS, DenoiseError};
use jobs::{Admission, JobTable, MAX_PENDING_PER_OWNER};
use worker::{BakeJob, LoadedDenoiser};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DenoiseStatus {
    Disabled,
    ModelMissing,
    Idle,
    Queued,
    Running {
        done: usize,
        total: usize,
    },
    Ready,
    Unsupported,
    Failed {
        #[serde(serialize_with = "user_message")]
        error: DenoiseError,
    },
}

impl DenoiseStatus {
    fn settled(&self) -> bool {
        matches!(self, Self::Ready | Self::Unsupported | Self::Failed { .. })
    }
}

fn user_message<S: Serializer>(error: &DenoiseError, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(error.user_message())
}

pub enum ExportPlane {
    Ready(Arc<DenoisePlane>),
    Disabled,
    ModelMissing,
    Unsupported,
}

enum Subscription {
    Settled(DenoiseStatus),
    Watch(watch::Receiver<DenoiseStatus>),
}

#[derive(Clone)]
pub struct DenoiseService {
    render: RenderService,
    models: ModelStore,
    mode: MlRuntimeMode,
    idle: Duration,
    export_wait: Duration,
    jobs: JobTable,
    worker: Arc<Semaphore>,
    permits: InferencePermits,
    loaded: Arc<Mutex<Option<LoadedDenoiser>>>,
}

impl DenoiseService {
    pub fn new(
        config: &Config,
        render: RenderService,
        models: ModelStore,
        permits: InferencePermits,
    ) -> Self {
        Self {
            render,
            models,
            mode: config.ml_runtime,
            idle: Duration::from_secs(config.ml_idle_secs),
            export_wait: Duration::from_secs(config.export_timeout_secs),
            jobs: JobTable::default(),
            worker: Arc::new(Semaphore::new(1)),
            permits,
            loaded: Arc::new(Mutex::new(None)),
        }
    }

    fn enabled(&self) -> bool {
        self.mode != MlRuntimeMode::Off
    }

    async fn key(&self, identity: RenderIdentity, source: Uuid) -> Result<PlaneKey, DenoiseStatus> {
        if !self.enabled() {
            return Err(DenoiseStatus::Disabled);
        }
        let model = self
            .render
            .denoise()
            .model()
            .await
            .ok_or(DenoiseStatus::ModelMissing)?;
        Ok(PlaneKey::new(identity, source, model))
    }

    pub async fn status(&self, identity: RenderIdentity, source: Uuid) -> DenoiseStatus {
        let key = match self.key(identity, source).await {
            Ok(key) => key,
            Err(status) => return status,
        };
        if let Some(state) = self.jobs.state(&key) {
            return state;
        }
        if self.render.denoise().store().contains(&key).await {
            return DenoiseStatus::Ready;
        }
        DenoiseStatus::Idle
    }

    pub async fn start(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<DenoiseStatus, DenoiseError> {
        let subscription = self
            .subscribe(identity, immich, source, Some(MAX_PENDING_PER_OWNER))
            .await?;
        Ok(match subscription {
            Subscription::Settled(status) => status,
            Subscription::Watch(rx) => rx.borrow().clone(),
        })
    }

    async fn subscribe(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        owner_limit: Option<usize>,
    ) -> Result<Subscription, DenoiseError> {
        let key = match self.key(identity, source).await {
            Ok(key) => key,
            Err(status) => return Ok(Subscription::Settled(status)),
        };
        let store = self.render.denoise().store();
        if store.contains(&key).await {
            return Ok(Subscription::Settled(DenoiseStatus::Ready));
        }
        let purge = store.ticket(key.owner).await;
        let (ticket, rx) = match self.jobs.admit(&key, owner_limit) {
            Admission::Existing(rx) => return Ok(Subscription::Watch(rx)),
            Admission::Busy => return Err(DenoiseError::Busy),
            Admission::New(ticket, rx) => (ticket, rx),
        };
        let job = BakeJob {
            service: self.clone(),
            key,
            identity,
            immich: immich.clone(),
            purge,
            ticket,
        };
        tokio::spawn(job.run());
        Ok(Subscription::Watch(rx))
    }

    async fn stored_plane(
        &self,
        identity: RenderIdentity,
        source: Uuid,
    ) -> Option<Arc<DenoisePlane>> {
        let inputs = self.render.denoise();
        let key = PlaneKey::new(identity, source, inputs.model().await?);
        inputs.store().get(&key).await
    }

    pub async fn export_plane(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<ExportPlane, DenoiseError> {
        if let Some(plane) = self.stored_plane(identity, source).await {
            return Ok(ExportPlane::Ready(plane));
        }
        let status = match self.subscribe(identity, immich, source, None).await? {
            Subscription::Settled(status) => status,
            Subscription::Watch(rx) => self.wait(rx, source).await?,
        };
        match status {
            DenoiseStatus::Disabled => Ok(ExportPlane::Disabled),
            DenoiseStatus::ModelMissing => Ok(ExportPlane::ModelMissing),
            DenoiseStatus::Unsupported => Ok(ExportPlane::Unsupported),
            DenoiseStatus::Failed { error } => Err(error),
            DenoiseStatus::Idle
            | DenoiseStatus::Queued
            | DenoiseStatus::Running { .. }
            | DenoiseStatus::Ready => self
                .stored_plane(identity, source)
                .await
                .map(ExportPlane::Ready)
                .ok_or(DenoiseError::Purged),
        }
    }

    async fn wait(
        &self,
        mut rx: watch::Receiver<DenoiseStatus>,
        source: Uuid,
    ) -> Result<DenoiseStatus, DenoiseError> {
        match tokio::time::timeout(self.export_wait, rx.wait_for(DenoiseStatus::settled)).await {
            Ok(Ok(status)) => Ok(status.clone()),
            Ok(Err(e)) => Err(DenoiseError::internal("waiting for the AI denoise bake", e)),
            Err(_) => {
                tracing::warn!(
                    %source,
                    waited_secs = self.export_wait.as_secs(),
                    "export gave up waiting for the AI denoise bake"
                );
                Err(DenoiseError::StillRunning)
            }
        }
    }

    pub fn purge_owner(&self, owner: Uuid) {
        self.jobs.drop_owner(owner);
    }

    pub fn purge_all(&self) {
        self.jobs.clear();
    }

    pub async fn release_idle(&self) {
        if let Ok(_permit) = self.worker.try_acquire() {
            let mut loaded = self.loaded.lock().await;
            if loaded.as_ref().is_some_and(|l| l.idle_for() > self.idle) {
                *loaded = None;
            }
        }
    }
}
