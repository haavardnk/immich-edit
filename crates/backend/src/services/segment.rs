use std::sync::Arc;
use std::time::{Duration, Instant};

use segment::runtime::SessionConfig;
use segment::{BakeParams, ModelKind, RuntimeMode, Segmenter, bake, catalog};
use tokio::sync::{Mutex, Semaphore};

use crate::config::{Config, SegmentRuntimeMode};
use crate::services::model_store::{ModelStore, ModelStoreError};

#[derive(Debug, thiserror::Error)]
pub enum SegmentServiceError {
    #[error("segmentation is disabled")]
    Disabled,
    #[error("no model installed for {0}")]
    ModelMissing(&'static str),
    #[error("model store: {0}")]
    Store(#[from] ModelStoreError),
    #[error("inference: {0}")]
    Inference(String),
    #[error("worker crashed")]
    Worker,
}

#[derive(Debug, Clone)]
pub struct MaskResult {
    pub bytes: Vec<u8>,
    pub prob: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub backend: &'static str,
    pub model_id: String,
    pub elapsed_ms: u64,
}

fn luma_guide(rgb8: &[u8]) -> Vec<f32> {
    rgb8.chunks_exact(3)
        .map(|p| (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) / 255.0)
        .collect()
}

struct Loaded {
    catalog_id: String,
    segmenter: Segmenter,
    used_at: Instant,
}

#[derive(Clone)]
pub struct SegmentService {
    models: ModelStore,
    mode: SegmentRuntimeMode,
    max_edge: u32,
    idle: Duration,
    permits: Arc<Semaphore>,
    loaded: Arc<Mutex<Option<Loaded>>>,
}

impl SegmentService {
    pub fn new(config: &Config, models: ModelStore) -> Self {
        Self {
            models,
            mode: config.segment_runtime,
            max_edge: config.segment_max_edge,
            idle: Duration::from_secs(config.segment_idle_secs),
            permits: Arc::new(Semaphore::new(config.segment_max_concurrency)),
            loaded: Arc::new(Mutex::new(None)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.mode != SegmentRuntimeMode::Off
    }

    pub fn max_edge(&self) -> u32 {
        self.max_edge
    }

    fn runtime_mode(&self) -> RuntimeMode {
        match self.mode {
            SegmentRuntimeMode::Gpu => RuntimeMode::Gpu,
            SegmentRuntimeMode::Cpu => RuntimeMode::Cpu,
            _ => RuntimeMode::Auto,
        }
    }

    pub async fn active_model(&self, kind: ModelKind) -> Option<String> {
        self.resolve(kind).await.ok()
    }

    async fn resolve(&self, kind: ModelKind) -> Result<String, SegmentServiceError> {
        if let Some(id) = self.models.preferred(kind.as_str()).await?
            && self.models.find_by_catalog(&id).await?.is_some()
        {
            return Ok(id);
        }
        if let Some(entry) = catalog::default_for(kind)
            && self.models.find_by_catalog(entry.id).await?.is_some()
        {
            return Ok(entry.id.to_string());
        }
        for entry in catalog::for_kind(kind) {
            if self.models.find_by_catalog(entry.id).await?.is_some() {
                return Ok(entry.id.to_string());
            }
        }
        Err(SegmentServiceError::ModelMissing(kind.as_str()))
    }

    pub async fn generate(
        &self,
        kind: ModelKind,
        rgb8: Vec<u8>,
        width: u32,
        height: u32,
        params: BakeParams,
    ) -> Result<MaskResult, SegmentServiceError> {
        if !self.enabled() {
            return Err(SegmentServiceError::Disabled);
        }
        let catalog_id = self.resolve(kind).await?;
        let path = self.models.resolve_path(&catalog_id).await?;
        let spec = catalog::find(&catalog_id)
            .ok_or(SegmentServiceError::ModelMissing(kind.as_str()))?
            .spec
            .clone();

        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| SegmentServiceError::Worker)?;

        let slot = self.loaded.clone();
        let mode = self.runtime_mode();
        let idle = self.idle;
        let id_for_task = catalog_id.clone();

        let started = Instant::now();
        let out = tokio::task::spawn_blocking(move || {
            let mut guard = slot.blocking_lock();
            let stale = guard
                .as_ref()
                .is_some_and(|l| l.catalog_id != id_for_task || l.used_at.elapsed() > idle);
            if stale {
                *guard = None;
            }
            if guard.is_none() {
                let segmenter = Segmenter::open(&path, spec, mode, &SessionConfig::default())
                    .map_err(|e| SegmentServiceError::Inference(e.to_string()))?;
                *guard = Some(Loaded {
                    catalog_id: id_for_task.clone(),
                    segmenter,
                    used_at: Instant::now(),
                });
            }
            let loaded = guard.as_mut().ok_or(SegmentServiceError::Worker)?;
            let mask = loaded
                .segmenter
                .run(&rgb8, width, height)
                .map_err(|e| SegmentServiceError::Inference(e.to_string()))?;
            loaded.used_at = Instant::now();
            let backend = loaded.segmenter.backend().as_str();

            let guide = luma_guide(&rgb8);
            let bytes = bake(
                &mask.values,
                &guide,
                width as usize,
                height as usize,
                params,
            );
            let prob: Vec<u8> = mask
                .values
                .iter()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
                .collect();
            Ok::<(Vec<u8>, Vec<u8>, &'static str), SegmentServiceError>((bytes, prob, backend))
        })
        .await
        .map_err(|_| SegmentServiceError::Worker)??;

        let elapsed_ms = started.elapsed().as_millis() as u64;
        tracing::info!(
            model = %catalog_id,
            backend = out.2,
            width,
            height,
            elapsed_ms,
            "generated mask"
        );

        Ok(MaskResult {
            bytes: out.0,
            prob: out.1,
            width,
            height,
            backend: out.2,
            model_id: catalog_id,
            elapsed_ms,
        })
    }

    pub async fn rebake(
        &self,
        prob: Vec<u8>,
        rgb8: Vec<u8>,
        width: u32,
        height: u32,
        params: BakeParams,
    ) -> Result<Vec<u8>, SegmentServiceError> {
        tokio::task::spawn_blocking(move || {
            let values: Vec<f32> = prob.iter().map(|v| *v as f32 / 255.0).collect();
            let guide = luma_guide(&rgb8);
            bake(&values, &guide, width as usize, height as usize, params)
        })
        .await
        .map_err(|_| SegmentServiceError::Worker)
    }

    pub async fn release_idle(&self) {
        let mut guard = self.loaded.lock().await;
        let expired = guard
            .as_ref()
            .is_some_and(|l| l.used_at.elapsed() > self.idle);
        if expired {
            *guard = None;
        }
    }
}
