use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{OutputFormat, PreviewMode, RenderOptions};
use tokio::fs;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::services::render::{RenderError, RenderService};

const TTL: Duration = Duration::from_secs(60 * 60 * 24 * 30);

#[derive(Debug, thiserror::Error)]
pub enum EditedThumbError {
    #[error("not found")]
    NotFound,
    #[error("hash mismatch")]
    HashMismatch,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("render: {0}")]
    Render(#[from] RenderError),
}

#[derive(Clone)]
pub struct EditedThumbService {
    dir: Arc<PathBuf>,
    semaphore: Arc<Semaphore>,
}

impl EditedThumbService {
    pub fn new(cache_dir: &Path, max_concurrency: usize) -> std::io::Result<Self> {
        let dir = cache_dir.join("edited-thumb");
        std::fs::create_dir_all(&dir)?;
        let svc = Self {
            dir: Arc::new(dir),
            semaphore: Arc::new(Semaphore::new(max_concurrency.max(1))),
        };
        svc.sweep_blocking();
        Ok(svc)
    }

    fn sweep_blocking(&self) {
        let dir = self.dir.clone();
        std::thread::spawn(move || {
            let now = SystemTime::now();
            let Ok(read) = std::fs::read_dir(dir.as_path()) else {
                return;
            };
            for entry in read.flatten() {
                let Ok(meta) = entry.metadata() else {
                    continue;
                };
                let Ok(modified) = meta.modified() else {
                    continue;
                };
                if now.duration_since(modified).unwrap_or_default() > TTL {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        });
    }

    fn cache_path(&self, asset_id: Uuid, hash: &str, size: u32) -> PathBuf {
        self.dir.join(format!("{asset_id}-{hash}-{size}.jpg"))
    }

    pub async fn get_or_render(
        &self,
        render: &RenderService,
        asset_id: Uuid,
        edits: Edits,
        expected_hash: &str,
        size: u32,
    ) -> Result<Vec<u8>, EditedThumbError> {
        let actual = edits.stable_hash();
        if actual != expected_hash {
            return Err(EditedThumbError::HashMismatch);
        }
        let path = self.cache_path(asset_id, expected_hash, size);
        if let Ok(bytes) = fs::read(&path).await {
            return Ok(bytes);
        }
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| EditedThumbError::Io(std::io::ErrorKind::Other.into()))?;
        if let Ok(bytes) = fs::read(&path).await {
            return Ok(bytes);
        }
        let opts = RenderOptions {
            max_edge: size,
            quality: false,
            output: OutputFormat::Jpeg { quality: 80 },
            preview_mode: PreviewMode::None,
            ..Default::default()
        };
        let rendered = render.render(asset_id, edits, opts, None).await?;
        let tmp = path.with_extension("jpg.tmp");
        fs::write(&tmp, &rendered.bytes).await?;
        fs::rename(&tmp, &path).await?;
        Ok(rendered.bytes)
    }
}
