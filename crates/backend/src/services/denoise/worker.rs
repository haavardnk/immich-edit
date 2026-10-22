use std::time::{Duration, Instant};

use ml::Denoiser;
use ml::runtime::SessionConfig;
use raw_pipeline::PipelineError;

use super::jobs::JobTicket;
use super::{DenoiseError, DenoiseService, DenoiseStatus};
use crate::immich::ImmichClient;
use crate::services::denoise_store::{PlaneKey, PurgeTicket};
use crate::services::ml_runtime::runtime_mode;
use crate::services::render::RenderIdentity;
use crate::telemetry::ErrorChain;

pub(super) struct LoadedDenoiser {
    catalog_id: String,
    denoiser: Denoiser,
    used_at: Instant,
}

impl LoadedDenoiser {
    pub(super) fn idle_for(&self) -> Duration {
        self.used_at.elapsed()
    }
}

pub(super) struct BakeJob {
    pub service: DenoiseService,
    pub key: PlaneKey,
    pub identity: RenderIdentity,
    pub immich: ImmichClient,
    pub purge: PurgeTicket,
    pub ticket: JobTicket,
}

impl BakeJob {
    pub(super) async fn run(mut self) {
        let started = Instant::now();
        let state = match self.bake().await {
            Ok(backend) => {
                tracing::info!(
                    source = %self.key.source,
                    model = %self.key.model,
                    backend,
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "baked AI denoise"
                );
                DenoiseStatus::Ready
            }
            Err(DenoiseError::Unsupported(reason)) => {
                tracing::info!(source = %self.key.source, reason, "AI denoise unsupported");
                DenoiseStatus::Unsupported
            }
            Err(error) => {
                tracing::warn!(
                    source = %self.key.source,
                    model = %self.key.model,
                    error = %ErrorChain(&error),
                    "AI denoise bake failed"
                );
                DenoiseStatus::Failed { error }
            }
        };
        self.ticket.finish(state);
    }

    async fn bake(&self) -> Result<&'static str, DenoiseError> {
        let service = &self.service;
        let _worker = service
            .worker
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| DenoiseError::internal("waiting for the denoise worker", e))?;
        let store = service.render.denoise().store();
        if store.ticket(self.key.owner).await != self.purge {
            return Err(DenoiseError::Purged);
        }
        if store.contains(&self.key).await {
            return Ok("cache");
        }
        let progress = self.ticket.progress();
        progress.send_replace(DenoiseStatus::Running { done: 0, total: 0 });
        let frame = service
            .render
            .quality_frame(self.identity, &self.immich, self.key.source)
            .await?;
        if !frame.meta.is_raw {
            return Err(DenoiseError::Unsupported(
                "AI denoise works on RAW photos only".into(),
            ));
        }
        let path = service.models.resolve_path(&self.key.model).await?;
        let loaded = service.loaded.clone();
        let model = self.key.model.clone();
        let mode = runtime_mode(service.mode);
        let plane_id = self.key.plane_id();
        let permits = service.permits.clone();
        let purge = self.purge;
        let owner = self.key.owner;
        let generations = store.clone();
        let (plane, backend) = tokio::task::spawn_blocking(move || {
            let mut guard = loaded.blocking_lock();
            if guard.as_ref().is_some_and(|l| l.catalog_id != model) {
                *guard = None;
            }
            if guard.is_none() {
                let _permit = permits
                    .blocking_acquire()
                    .ok_or_else(|| DenoiseError::stopped("waiting for an inference permit"))?;
                let denoiser = Denoiser::open(&path, mode, &SessionConfig::default())
                    .map_err(|e| DenoiseError::internal("loading the denoise model", e))?;
                *guard = Some(LoadedDenoiser {
                    catalog_id: model,
                    denoiser,
                    used_at: Instant::now(),
                });
            }
            let loaded = guard
                .as_mut()
                .ok_or_else(|| DenoiseError::stopped("loading the denoise model"))?;
            let edge = loaded.denoiser.tile_edge();
            let denoiser = &mut loaded.denoiser;
            let mut failure: Option<DenoiseError> = None;
            let baked = raw_pipeline::ai_denoise::bake(
                &frame,
                plane_id,
                edge,
                |tile| {
                    let Some(_permit) = permits.blocking_acquire() else {
                        failure = Some(DenoiseError::stopped("waiting for an inference permit"));
                        return Err(PipelineError::Cancelled);
                    };
                    if generations.blocking_ticket(owner) != purge {
                        failure = Some(DenoiseError::Purged);
                        return Err(PipelineError::Cancelled);
                    }
                    denoiser.run(tile).map_err(|e| {
                        failure = Some(DenoiseError::internal("running the denoise model", e));
                        PipelineError::Cancelled
                    })
                },
                |done, total| {
                    progress.send_replace(DenoiseStatus::Running { done, total });
                },
            );
            let plane = baked.map_err(|e| failure.take().unwrap_or_else(|| e.into()))?;
            loaded.used_at = Instant::now();
            Ok::<_, DenoiseError>((plane, loaded.denoiser.backend().as_str()))
        })
        .await
        .map_err(|e| DenoiseError::internal("the denoise worker crashed", e))??;
        match store.put(self.key.clone(), plane, self.purge).await {
            Ok(Some(_)) => Ok(backend),
            Ok(None) => Err(DenoiseError::Purged),
            Err(e) => Err(DenoiseError::internal("storing the denoise plane", e)),
        }
    }
}
