use std::sync::Arc;

use crate::config::Config;
use crate::immich::ImmichClient;
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
}

impl AppState {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let immich = ImmichClient::new(config.immich_url.clone(), &config.immich_api_key)
            .map_err(|e| anyhow::anyhow!("immich client: {e}"))?;
        let edits = EditsStore::new(&config.cache_dir);
        let render = RenderService::new(immich.clone(), 8);
        let queue = RenderQueue::new(config.render_max_concurrency);
        Ok(Self {
            config: Arc::new(config),
            immich,
            edits,
            render,
            queue,
            preview_meta: PreviewMetaStore::new(),
        })
    }
}
