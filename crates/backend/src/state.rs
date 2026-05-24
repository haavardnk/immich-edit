use std::sync::Arc;

use crate::config::Config;
use crate::immich::ImmichClient;
use crate::services::edited_thumb::EditedThumbService;
use crate::services::edits_store::EditsStore;
use crate::services::preview_meta::PreviewMetaStore;
use crate::services::render::RenderService;
use crate::services::render_queue::RenderQueue;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub immich: ImmichClient,
    pub edits: EditsStore,
    pub render: RenderService,
    pub queue: RenderQueue,
    pub preview_meta: PreviewMetaStore,
    pub edited_thumb: EditedThumbService,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let immich = ImmichClient::new(config.immich_url.clone(), &config.immich_api_key)
            .map_err(|e| anyhow::anyhow!("immich client: {e}"))?;
        if let Some(parent) = std::path::Path::new(&config.cache_dir).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::create_dir_all(&config.cache_dir).ok();
        let edits = EditsStore::connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("edits store: {e}"))?;
        let render = RenderService::new(immich.clone(), 8, config.renderer);
        let queue = RenderQueue::new(config.render_max_concurrency);
        let edited_thumb =
            EditedThumbService::new(&config.cache_dir, config.render_max_concurrency)
                .map_err(|e| anyhow::anyhow!("edited thumb cache: {e}"))?;
        Ok(Self {
            config: Arc::new(config),
            immich,
            edits,
            render,
            queue,
            preview_meta: PreviewMetaStore::new(),
            edited_thumb,
        })
    }
}
