use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{JpegSubsampling, OutputFormat, PreviewMode, RenderOptions};
use tokio::fs;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::services::render::{RenderError, RenderIdentity, RenderService};

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
            let mut pending = vec![dir.as_ref().clone()];
            while let Some(current) = pending.pop() {
                let Ok(read) = std::fs::read_dir(&current) else {
                    continue;
                };
                for entry in read.flatten() {
                    let Ok(metadata) = entry.metadata() else {
                        continue;
                    };
                    if metadata.is_dir() {
                        pending.push(entry.path());
                        continue;
                    }
                    let Ok(modified) = metadata.modified() else {
                        continue;
                    };
                    if now.duration_since(modified).unwrap_or_default() > TTL {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        });
    }

    fn cache_path(
        &self,
        identity: RenderIdentity,
        asset_id: AssetKey,
        hash: &str,
        size: u32,
    ) -> PathBuf {
        self.dir
            .join(identity.server_epoch.to_string())
            .join(identity.owner.to_string())
            .join(format!("{asset_id}-{hash}-{size}.jpg"))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_or_render(
        &self,
        render: &RenderService,
        identity: RenderIdentity,
        immich: crate::immich::ImmichClient,
        asset_id: AssetKey,
        edits: Edits,
        expected_hash: &str,
        size: u32,
    ) -> Result<Vec<u8>, EditedThumbError> {
        let actual = edits.stable_hash();
        if actual != expected_hash {
            return Err(EditedThumbError::HashMismatch);
        }
        let dcp_revision = render.dcp_revision().await?;
        let render_hash = format!("{expected_hash}-{dcp_revision}");
        let path = self.cache_path(identity, asset_id, &render_hash, size);
        if let Ok(bytes) = fs::read(&path).await {
            return Ok(bytes);
        }
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| EditedThumbError::Io(std::io::ErrorKind::Other.into()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        if let Ok(bytes) = fs::read(&path).await {
            return Ok(bytes);
        }
        let opts = RenderOptions {
            max_edge: size,
            quality: false,
            output: OutputFormat::Jpeg {
                quality: 80,
                subsampling: JpegSubsampling::Chroma420,
            },
            preview_mode: PreviewMode::None,
            ..Default::default()
        };
        let rendered = render
            .render(identity, immich, asset_id.source(), edits, opts, None)
            .await?;
        let tmp = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        fs::write(&tmp, &rendered.bytes).await?;
        if let Err(error) = fs::rename(&tmp, &path).await {
            let _ = fs::remove_file(&tmp).await;
            return Err(error.into());
        }
        Ok(rendered.bytes)
    }

    pub async fn purge_asset(&self, server_epoch: i64, owner: Uuid, asset_id: AssetKey) {
        let dir = self
            .dir
            .join(server_epoch.to_string())
            .join(owner.to_string());
        let prefix = format!("{asset_id}-");
        let Ok(mut entries) = fs::read_dir(&dir).await else {
            return;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                let _ = fs::remove_file(entry.path()).await;
            }
        }
    }

    pub async fn purge_owner(&self, owner: Uuid) -> std::io::Result<()> {
        let mut epochs = fs::read_dir(self.dir.as_path()).await?;
        while let Some(epoch) = epochs.next_entry().await? {
            let path = epoch.path().join(owner.to_string());
            if let Err(error) = fs::remove_dir_all(path).await
                && error.kind() != std::io::ErrorKind::NotFound
            {
                return Err(error);
            }
        }
        Ok(())
    }

    pub async fn purge_all(&self) -> std::io::Result<()> {
        if let Err(error) = fs::remove_dir_all(self.dir.as_path()).await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(error);
        }
        fs::create_dir_all(self.dir.as_path()).await
    }
}
