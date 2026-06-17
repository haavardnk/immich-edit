use std::sync::Arc;

use crate::config::Config;
use crate::immich::ImmichClient;
use crate::services::asset_counts::AssetCountCache;
use crate::services::edited_thumb::EditedThumbService;
use crate::services::edits_store::EditsStore;
use crate::services::job_store::JobStore;
use crate::services::preview_meta::PreviewMetaStore;
use crate::services::raster_store::RasterStore;
use crate::services::render::{RenderCacheOptions, RenderService};
use crate::services::render_queue::RenderQueue;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub immich: ImmichClient,
    pub edits: EditsStore,
    pub jobs: JobStore,
    pub render: RenderService,
    pub queue: RenderQueue,
    pub preview_meta: PreviewMetaStore,
    pub edited_thumb: EditedThumbService,
    pub rasters: RasterStore,
    pub tag_counts: AssetCountCache,
    pub people_counts: AssetCountCache,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let immich = ImmichClient::with_timeout(
            config.immich_url.clone(),
            &config.immich_api_key,
            std::time::Duration::from_secs(config.original_timeout_secs),
        )
        .map_err(|e| anyhow::anyhow!("immich client: {e}"))?;
        if let Some(parent) = std::path::Path::new(&config.cache_dir).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::create_dir_all(&config.cache_dir).ok();
        let edits = EditsStore::connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("edits store: {e}"))?;
        let jobs = JobStore::new(edits.pool());
        let rasters = RasterStore::new(&config.cache_dir, config.mask_cache_mb)
            .map_err(|e| anyhow::anyhow!("raster store: {e}"))?;
        let render = RenderService::new(
            immich.clone(),
            RenderCacheOptions {
                raw_frame_cache_mb: config.raw_frame_cache_mb,
                quality_frame_cache_mb: config.quality_frame_cache_mb,
                gpu_texture_cache_mb: config.gpu_texture_cache_mb,
            },
            config.renderer,
            rasters.clone(),
        );
        let queue = RenderQueue::new(config.render_max_concurrency);
        let edited_thumb =
            EditedThumbService::new(&config.cache_dir, config.render_max_concurrency)
                .map_err(|e| anyhow::anyhow!("edited thumb cache: {e}"))?;
        Ok(Self {
            config: Arc::new(config),
            immich,
            edits,
            jobs,
            render,
            queue,
            preview_meta: PreviewMetaStore::new(),
            edited_thumb,
            rasters,
            tag_counts: AssetCountCache::new("tagIds"),
            people_counts: AssetCountCache::new("personIds"),
        })
    }
}
