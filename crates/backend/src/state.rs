use std::sync::Arc;

use crate::config::Config;
use crate::immich::ImmichClient;
use crate::services::asset_counts::AssetCountCache;
use crate::services::auth_store::AuthStore;
use crate::services::crypto::InstanceCrypto;
use crate::services::dcp_store::DcpStore;
use crate::services::edited_thumb::EditedThumbService;
use crate::services::edits_store::EditsStore;
use crate::services::instance_store::InstanceStore;
use crate::services::job_store::JobStore;
use crate::services::login_limiter::LoginLimiter;
use crate::services::lut_store::LutStore;
use crate::services::preview_meta::PreviewMetaStore;
use crate::services::raster_store::RasterStore;
use crate::services::render::{RenderCacheOptions, RenderService};
use crate::services::render_queue::RenderQueue;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub crypto: Arc<InstanceCrypto>,
    pub instance: InstanceStore,
    pub auth: AuthStore,
    pub login_limiter: Arc<LoginLimiter>,
    pub immich: ImmichClient,
    pub edits: EditsStore,
    pub jobs: JobStore,
    pub render: RenderService,
    pub queue: RenderQueue,
    pub preview_meta: PreviewMetaStore,
    pub edited_thumb: EditedThumbService,
    pub rasters: RasterStore,
    pub luts: LutStore,
    pub dcp: DcpStore,
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
        let instance = InstanceStore::new(edits.pool());
        let has_secrets = instance.has_encrypted_secrets().await.unwrap_or(false);
        let key_path = std::path::Path::new(&config.cache_dir).join("instance.key");
        let crypto = Arc::new(
            InstanceCrypto::load_or_create(&key_path, has_secrets)
                .map_err(|e| anyhow::anyhow!("instance key: {e}"))?,
        );
        let auth = AuthStore::new(edits.pool(), crypto.clone());
        let login_limiter = Arc::new(LoginLimiter::new());
        let jobs = JobStore::new(edits.pool());
        let rasters = RasterStore::new(&config.cache_dir, config.mask_cache_mb)
            .map_err(|e| anyhow::anyhow!("raster store: {e}"))?;
        let luts = LutStore::new(edits.pool(), std::path::Path::new(&config.cache_dir))
            .map_err(|e| anyhow::anyhow!("lut store: {e}"))?;
        let dcp = DcpStore::new(edits.pool(), std::path::Path::new(&config.cache_dir))
            .map_err(|e| anyhow::anyhow!("dcp store: {e}"))?;
        let dcp_dir = std::env::var("DCP_DIR").unwrap_or_else(|_| "./assets/dcp".into());
        let dcp_dir_path = std::path::Path::new(&dcp_dir);
        if !dcp_dir_path.exists() {
            tracing::warn!(dir = %dcp_dir, "DCP_DIR not found; no bundled camera profiles will be imported");
        }
        let imported = dcp
            .import_bundled(dcp_dir_path)
            .await
            .map_err(|e| anyhow::anyhow!("dcp bundle import: {e}"))?;
        if imported > 0 {
            tracing::info!(count = imported, "imported bundled dcp profiles");
        }
        let render = RenderService::new(
            RenderCacheOptions {
                raw_frame_cache_mb: config.raw_frame_cache_mb,
                quality_frame_cache_mb: config.quality_frame_cache_mb,
                gpu_texture_cache_mb: config.gpu_texture_cache_mb,
            },
            config.renderer,
            rasters.clone(),
            luts.clone(),
            dcp.clone(),
        );
        let queue = RenderQueue::new(config.render_max_concurrency);
        let edited_thumb =
            EditedThumbService::new(&config.cache_dir, config.render_max_concurrency)
                .map_err(|e| anyhow::anyhow!("edited thumb cache: {e}"))?;
        Ok(Self {
            config: Arc::new(config),
            crypto,
            instance,
            auth,
            login_limiter,
            immich,
            edits,
            jobs,
            render,
            queue,
            preview_meta: PreviewMetaStore::new(),
            edited_thumb,
            rasters,
            luts,
            dcp,
            tag_counts: AssetCountCache::new("tagIds"),
            people_counts: AssetCountCache::new("personIds"),
        })
    }
}
