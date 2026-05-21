use std::io;
use std::path::{Path, PathBuf};

use raw_pipeline::edits::{Edits, Sidecar};
use tokio::fs;
use uuid::Uuid;

pub const RENDERER_VERSION: &str = "cpu-0.1.0";

#[derive(Debug, thiserror::Error)]
pub enum EditsStoreError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct EditsStore {
    root: PathBuf,
}

impl EditsStore {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            root: cache_dir.as_ref().join("edits"),
        }
    }

    fn path_for(&self, asset_id: Uuid) -> PathBuf {
        self.root.join(format!("{asset_id}.json"))
    }

    pub async fn ensure_root(&self) -> Result<(), EditsStoreError> {
        fs::create_dir_all(&self.root).await?;
        Ok(())
    }

    pub async fn get(&self, asset_id: Uuid) -> Result<Option<Sidecar>, EditsStoreError> {
        let path = self.path_for(asset_id);
        match fs::read(&path).await {
            Ok(bytes) => {
                let sidecar: Sidecar = serde_json::from_slice(&bytes)?;
                Ok(Some(sidecar))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(EditsStoreError::Io(e)),
        }
    }

    pub async fn get_edits_or_default(&self, asset_id: Uuid) -> Result<Edits, EditsStoreError> {
        Ok(self.get(asset_id).await?.map(|s| s.edits).unwrap_or_default())
    }

    pub async fn put(
        &self,
        asset_id: Uuid,
        edits: Edits,
        immich_updated_at: Option<String>,
        immich_checksum: Option<String>,
    ) -> Result<Sidecar, EditsStoreError> {
        self.ensure_root().await?;
        let now = chrono_like_now();
        let sidecar = Sidecar {
            schema_version: 1,
            asset_id,
            immich_updated_at,
            immich_checksum,
            renderer_version: RENDERER_VERSION.into(),
            edits: edits.clamped(),
            updated_at: now,
        };
        let bytes = serde_json::to_vec_pretty(&sidecar)?;
        let final_path = self.path_for(asset_id);
        let tmp_path = self
            .root
            .join(format!(".{asset_id}.{}.tmp", Uuid::new_v4()));
        {
            let f = fs::File::create(&tmp_path).await?;
            let mut writer = tokio::io::BufWriter::new(f);
            use tokio::io::AsyncWriteExt;
            writer.write_all(&bytes).await?;
            writer.flush().await?;
            writer.get_ref().sync_all().await?;
        }
        fs::rename(&tmp_path, &final_path).await?;
        Ok(sidecar)
    }

    pub async fn delete(&self, asset_id: Uuid) -> Result<bool, EditsStoreError> {
        let path = self.path_for(asset_id);
        match fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(EditsStoreError::Io(e)),
        }
    }
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_pipeline::edits::Edits;
    use tempfile::tempdir;

    fn uid() -> Uuid {
        Uuid::new_v4()
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let dir = tempdir().unwrap();
        let store = EditsStore::new(dir.path());
        let out = store.get(uid()).await.unwrap();
        if out.is_some() {
            panic!("expected none");
        }
    }

    #[tokio::test]
    async fn put_then_get_roundtrips() {
        let dir = tempdir().unwrap();
        let store = EditsStore::new(dir.path());
        let id = uid();
        let edits = Edits {
            exposure_ev: 1.0,
            rotate: 90,
            ..Default::default()
        };
        let saved = store
            .put(
                id,
                edits.clone(),
                Some("2026-01-01T00:00:00Z".into()),
                Some("abc".into()),
            )
            .await
            .unwrap();
        if saved.asset_id != id {
            panic!("id");
        }
        let loaded = store.get(id).await.unwrap().unwrap();
        if loaded.edits.exposure_ev != 1.0 || loaded.edits.rotate != 90 {
            panic!("edits");
        }
        if loaded.immich_checksum.as_deref() != Some("abc") {
            panic!("checksum");
        }
    }

    #[tokio::test]
    async fn put_clamps_invalid_values() {
        let dir = tempdir().unwrap();
        let store = EditsStore::new(dir.path());
        let id = uid();
        let edits = Edits {
            exposure_ev: 99.0,
            rotate: 33,
            ..Default::default()
        };
        let saved = store.put(id, edits, None, None).await.unwrap();
        if saved.edits.exposure_ev > 5.0 {
            panic!("not clamped: {}", saved.edits.exposure_ev);
        }
        if saved.edits.rotate != 0 {
            panic!("rotate not snapped: {}", saved.edits.rotate);
        }
    }

    #[tokio::test]
    async fn delete_removes() {
        let dir = tempdir().unwrap();
        let store = EditsStore::new(dir.path());
        let id = uid();
        store.put(id, Edits::default(), None, None).await.unwrap();
        if !store.delete(id).await.unwrap() {
            panic!("first delete");
        }
        if store.delete(id).await.unwrap() {
            panic!("second delete should be false");
        }
        if store.get(id).await.unwrap().is_some() {
            panic!("still present");
        }
    }

    #[tokio::test]
    async fn atomic_write_no_temp_left() {
        let dir = tempdir().unwrap();
        let store = EditsStore::new(dir.path());
        let id = uid();
        store.put(id, Edits::default(), None, None).await.unwrap();
        let mut found_tmp = false;
        let mut entries = fs::read_dir(dir.path().join("edits")).await.unwrap();
        while let Some(e) = entries.next_entry().await.unwrap() {
            let name = e.file_name().into_string().unwrap();
            if name.ends_with(".tmp") {
                found_tmp = true;
            }
        }
        if found_tmp {
            panic!("tmp file left behind");
        }
    }
}
